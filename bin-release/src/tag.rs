use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::git;
use crate::metadata::{self, ReleaseInfo, ReleaseSource};
use crate::signing;

const GATEWAY_REPOSITORY: &str =
    "https://github.com/lingting-projects/lingting-ai-gateway.git";

const FRAMEWORK_REPOSITORY: &str =
    "https://github.com/lingting-projects/lingting-rust-framework.git";

const REACT_UI_REPOSITORY: &str =
    "https://github.com/lingting/lingting-react-ui.git";

const SIGN_KEY_ENV: &str = "LINGTING_RELEASE_SIGN_KEY";

pub fn run() -> Result<()> {
    let root = git::repository_root()?;

    verify_gateway(&root)?;

    let framework_dir = root
        .parent()
        .context("gateway repository has no parent directory")?
        .join("lingting-rust-framework");

    let react_ui_dir = resolve_react_ui_repository(&root)?;

    let framework = verify_framework_repository(&framework_dir)?;
    let react_ui = verify_react_ui_repository(&react_ui_dir)?;

    verify_cargo_metadata(&root)?;

    let version = cargo_version(&root)?;
    let tag = format!("v{version}");

    verify_tag_does_not_exist(&root, &tag)?;

    let framework_commands =
        generate_framework_commands(&root, &framework)?;

    update_release_script(&root, &framework_commands)?;

    let info = ReleaseInfo::new(tag.clone(), framework, react_ui);

    let info_path = metadata::info_path(&root);
    let signature_path = metadata::signature_path(&root);
    let public_key_path = metadata::public_key_path(&root);

    info.write(&info_path)?;

    let sign_key = signing_key()?;

    signing::sign(&info_path, &signature_path, &sign_key)?;
    signing::verify(
        &info_path,
        &signature_path,
        &public_key_path,
    )?;

    verify_release_files(&root)?;

    commit_metadata(&root)?;
    create_tag(&root, &tag)?;
    push_release(&root, &tag)?;

    println!("release tag created and pushed: {tag}");

    Ok(())
}

fn verify_gateway(root: &Path) -> Result<()> {
    git::ensure_origin(root, GATEWAY_REPOSITORY)?;
    git::require_clean_tree(root)?;

    let branch = git::current_branch(root)?;

    if branch.is_empty() {
        bail!("gateway is in detached HEAD state");
    }

    Ok(())
}

fn verify_framework_repository(
    repository: &Path,
) -> Result<ReleaseSource> {
    if !repository.is_dir() {
        bail!(
            "framework repository does not exist: {}",
            repository.display()
        );
    }

    git::ensure_origin(repository, FRAMEWORK_REPOSITORY)?;
    git::require_clean_tree(repository)?;

    let branch = git::current_branch(repository)?;

    if branch.is_empty() {
        bail!("framework repository is in detached HEAD state");
    }

    let commit = git::head_commit(repository)?;

    validate_commit(&commit, "framework commit")?;

    Ok(ReleaseSource {
        repository: FRAMEWORK_REPOSITORY.to_owned(),
        branch,
        commit,
    })
}

fn verify_react_ui_repository(repository: &Path) -> Result<ReleaseSource> {
    if !repository.is_dir() {
        bail!(
            "react-ui repository does not exist: {}",
            repository.display()
        );
    }

    git::ensure_origin(repository, REACT_UI_REPOSITORY)?;
    git::require_clean_tree(repository)?;

    let branch = git::current_branch(repository)?;

    if branch.is_empty() {
        bail!("react-ui repository is in detached HEAD state");
    }

    let commit = git::head_commit(repository)?;

    validate_commit(&commit, "react-ui commit")?;

    Ok(ReleaseSource {
        repository: REACT_UI_REPOSITORY.to_owned(),
        branch,
        commit,
    })
}

fn validate_commit(commit: &str, name: &str) -> Result<()> {
    if commit.len() != 40 || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("invalid {name}: {commit}");
    }

    Ok(())
}

fn resolve_react_ui_repository(root: &Path) -> Result<PathBuf> {
    let lri = root.join("lingting-ai-gateway-ui").join("lri");

    if !lri.exists() {
        bail!("lri does not exist: {}", lri.display());
    }

    let real_lri = fs::canonicalize(&lri)
        .with_context(|| format!("failed to resolve {}", lri.display()))?;

    if real_lri.file_name().and_then(|name| name.to_str()) != Some("src") {
        bail!(
            "lri does not resolve to a react-ui src directory: {}",
            real_lri.display()
        );
    }

    let repository = real_lri
        .parent()
        .context("react-ui src directory has no parent")?
        .to_path_buf();

    if !repository.join(".git").exists() {
        bail!(
            "resolved react-ui directory is not a Git repository: {}",
            repository.display()
        );
    }

    Ok(repository)
}

fn verify_cargo_metadata(root: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--locked",
            "--format-version",
            "1",
        ])
        .current_dir(root)
        .output()
        .context("failed to execute cargo metadata")?;

    if !output.status.success() {
        bail!(
            "cargo metadata verification failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}

fn cargo_version(root: &Path) -> Result<String> {
    let cargo_toml = root.join("Cargo.toml");

    let content = fs::read_to_string(&cargo_toml)
        .with_context(|| format!("failed to read {}", cargo_toml.display()))?;

    if let Some(version) =
        find_section_version(&content, "[workspace.package]")
    {
        return Ok(version);
    }

    if let Some(version) = find_section_version(&content, "[package]") {
        return Ok(version);
    }

    bail!(
        "failed to find version in {}",
        cargo_toml.display()
    )
}

fn find_section_version(content: &str, section_name: &str) -> Option<String> {
    let mut in_section = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') && line.ends_with(']') {
            in_section = line == section_name;
            continue;
        }

        if !in_section || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        if key.trim() != "version" {
            continue;
        }

        let value = value
            .split('#')
            .next()
            .unwrap_or(value)
            .trim()
            .trim_matches('"');

        if !value.is_empty() {
            return Some(value.to_owned());
        }
    }

    None
}

fn generate_framework_commands(
    root: &Path,
    framework: &ReleaseSource,
) -> Result<Vec<String>> {
    let cargo_toml = root.join("Cargo.toml");

    let content = fs::read_to_string(&cargo_toml)
        .with_context(|| format!("failed to read {}", cargo_toml.display()))?;

    let mut in_workspace_dependencies = false;
    let mut framework_lines = Vec::new();
    let mut framework_packages = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_dependencies =
                trimmed == "[workspace.dependencies]";
            continue;
        }

        if !in_workspace_dependencies
            || !trimmed.starts_with("framework-")
        {
            continue;
        }

        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };

        let name = name.trim();

        if !name.starts_with("framework-") {
            continue;
        }

        framework_lines.push(line.to_owned());
        framework_packages.push(name.to_owned());
    }

    if framework_lines.is_empty() {
        bail!(
            "no framework-* dependencies found in [workspace.dependencies] of {}",
            cargo_toml.display()
        );
    }

    let mut commands = Vec::with_capacity(framework_lines.len() + 1);

    for line in framework_lines {
        commands.push(generate_toml_replace_command(
            &line,
            framework,
        ));
    }

    framework_packages.sort();
    framework_packages.dedup();

    commands.push(generate_cargo_update_command(
        &framework_packages,
    ));

    Ok(commands)
}

fn generate_toml_replace_command(
    original_line: &str,
    framework: &ReleaseSource,
) -> String {
    let replacement = build_git_dependency_line(
        original_line,
        framework,
    );

    format!(
        "python3 - \"$ROOT_DIR/Cargo.toml\" \"{}\" \"{}\" <<'PY'\n\
import pathlib\n\
import sys\n\
\n\
cargo_toml = pathlib.Path(sys.argv[1])\n\
old_line = sys.argv[2]\n\
new_line = sys.argv[3]\n\
content = cargo_toml.read_text()\n\
lines = content.splitlines(keepends=True)\n\
replaced = False\n\
for index, line in enumerate(lines):\n\
    if line.rstrip(\"\\r\\n\") == old_line:\n\
        newline = \"\\r\\n\" if line.endswith(\"\\r\\n\") else \"\\n\" if line.endswith(\"\\n\") else \"\"\n\
        lines[index] = new_line + newline\n\
        replaced = True\n\
        break\n\
if not replaced:\n\
    raise SystemExit(\"framework dependency line not found: \" + old_line)\n\
cargo_toml.write_text(\"\".join(lines))\n\
PY",
        shell_single_quote(original_line),
        shell_single_quote(&replacement),
    )
}

fn build_git_dependency_line(
    original_line: &str,
    framework: &ReleaseSource,
) -> String {
    let leading_len = original_line
        .len()
        - original_line
        .trim_start_matches(char::is_whitespace)
        .len();

    let leading = &original_line[..leading_len];
    let trimmed = original_line.trim();

    let Some((name, value)) = trimmed.split_once('=') else {
        return original_line.to_owned();
    };

    let name = name.trim();
    let value = value.trim();

    let features = extract_inline_attribute(value, "features");
    let default_features =
        extract_inline_attribute(value, "default-features");

    let mut dependency = format!(
        "{} = {{ git = \"{}\", branch = \"{}\", rev = \"{}\"",
        name,
        framework.repository,
        framework.branch,
        framework.commit,
    );

    if let Some(value) = default_features {
        dependency.push_str(", default-features = ");
        dependency.push_str(&value);
    }

    if let Some(value) = features {
        dependency.push_str(", features = ");
        dependency.push_str(&value);
    }

    dependency.push_str(" }");

    format!("{leading}{dependency}")
}

fn extract_inline_attribute(
    value: &str,
    attribute: &str,
) -> Option<String> {
    let marker = format!("{attribute}");

    let mut search_start = 0usize;

    while let Some(relative_start) =
        value[search_start..].find(&marker)
    {
        let start = search_start + relative_start;
        let after_name = &value[start + marker.len()..];

        if !after_name
            .chars()
            .next()
            .is_some_and(|character| {
                character.is_whitespace() || character == '='
            })
        {
            search_start = start + marker.len();
            continue;
        }

        let after_equals = after_name.trim_start();

        let Some(after_equals) = after_equals.strip_prefix('=') else {
            search_start = start + marker.len();
            continue;
        };

        let after_equals = after_equals.trim_start();

        let mut depth = 0usize;
        let mut in_string = false;
        let mut escaped = false;

        for (offset, character) in after_equals.char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                    continue;
                }

                match character {
                    '\\' => escaped = true,
                    '"' => in_string = false,
                    _ => {}
                }

                continue;
            }

            match character {
                '"' => in_string = true,
                '[' | '{' | '(' => depth += 1,
                ']' | '}' | ')' => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                ',' if depth == 0 => {
                    return Some(
                        after_equals[..offset]
                            .trim()
                            .to_owned(),
                    );
                }
                _ => {}
            }
        }

        return Some(
            after_equals
                .trim()
                .trim_end_matches('}')
                .trim()
                .to_owned(),
        );
    }

    None
}

fn generate_cargo_update_command(packages: &[String]) -> String {
    let mut command = String::from("cargo update");

    for package in packages {
        command.push_str(" -p ");
        command.push_str(&shell_word(package));
    }

    command
}

fn shell_word(value: &str) -> String {
    if value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric()
            || matches!(byte, b'_' | b'-' | b'.' | b'/')
    }) {
        value.to_owned()
    } else {
        shell_single_quote(value)
    }
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn update_release_script(
    root: &Path,
    framework_commands: &[String],
) -> Result<()> {
    let release_script = metadata::release_dir(root).join("release.sh");

    let original = fs::read_to_string(&release_script)
        .with_context(|| {
            format!(
                "failed to read {}",
                release_script.display()
            )
        })?;

    let function_start = original
        .find("rsync_framework()")
        .context("release.sh is missing rsync_framework function")?;

    let body_start = original[function_start..]
        .find('{')
        .map(|offset| function_start + offset + 1)
        .context("release.sh has an invalid rsync_framework function")?;

    let body_end = find_function_end(&original, body_start)
        .context("failed to locate rsync_framework function end")?;

    let mut function = String::from("rsync_framework() {\n");

    for command in framework_commands {
        for line in command.lines() {
            function.push_str("    ");
            function.push_str(line.trim_end());
            function.push('\n');
        }
    }

    function.push('}');

    let mut updated =
        String::with_capacity(original.len() + function.len());

    updated.push_str(&original[..function_start]);
    updated.push_str(&function);
    updated.push_str(&original[body_end..]);

    fs::write(&release_script, updated)
        .with_context(|| {
            format!(
                "failed to write {}",
                release_script.display()
            )
        })?;

    Ok(())
}

fn find_function_end(content: &str, body_start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut depth = 1usize;
    let mut index = body_start;
    let mut in_string = false;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];

        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }

            index += 1;
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;

                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }

        index += 1;
    }

    None
}

fn verify_tag_does_not_exist(root: &Path, tag: &str) -> Result<()> {
    let local = Command::new("git")
        .args([
            "rev-parse",
            "-q",
            "--verify",
            &format!("refs/tags/{tag}"),
        ])
        .current_dir(root)
        .output()
        .context("failed to check local release tag")?;

    if local.status.success() {
        bail!("release tag already exists locally: {tag}");
    }

    let remote = Command::new("git")
        .args([
            "ls-remote",
            "--exit-code",
            "--tags",
            "origin",
            &format!("refs/tags/{tag}"),
        ])
        .current_dir(root)
        .output()
        .context("failed to check remote release tag")?;

    if remote.status.success() {
        bail!("release tag already exists remotely: {tag}");
    }

    Ok(())
}

fn signing_key() -> Result<PathBuf> {
    if let Ok(path) = std::env::var(SIGN_KEY_ENV) {
        let path = PathBuf::from(path);

        if path.is_file() {
            return Ok(path);
        }

        bail!(
            "{} points to a missing signing key: {}",
            SIGN_KEY_ENV,
            path.display()
        );
    }

    let home = dirs_home()?;
    let path = home
        .join(".ssh")
        .join("lingting_gateway_ed25519");

    if !path.is_file() {
        bail!(
            "release signing key does not exist: {}",
            path.display()
        );
    }

    Ok(path)
}

fn dirs_home() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home));
    }

    if let Some(home) = std::env::var_os("USERPROFILE") {
        return Ok(PathBuf::from(home));
    }

    bail!("unable to determine home directory")
}

fn verify_release_files(root: &Path) -> Result<()> {
    for path in [
        metadata::info_path(root),
        metadata::signature_path(root),
        metadata::public_key_path(root),
    ] {
        if !path.is_file() {
            bail!(
                "required release file does not exist: {}",
                path.display()
            );
        }
    }

    Ok(())
}

fn commit_metadata(root: &Path) -> Result<()> {
    git::run(
        root,
        &[
            "add",
            ".release/release.sh",
            ".release/info",
            ".release/info.sig",
        ],
    )?;

    git::run(root, &["diff", "--cached", "--check"])?;

    let staged =
        git::output(root, &["diff", "--cached", "--name-only"])?;

    if staged.trim().is_empty() {
        bail!("release produced no Git changes");
    }

    git::run(
        root,
        &[
            "commit",
            "-m",
            "chore(release): update release metadata",
        ],
    )?;

    Ok(())
}

fn create_tag(root: &Path, tag: &str) -> Result<()> {
    git::run(
        root,
        &[
            "tag",
            "-a",
            tag,
            "-m",
            &format!("Release {tag}"),
        ],
    )?;

    Ok(())
}

fn push_release(root: &Path, tag: &str) -> Result<()> {
    let branch = git::current_branch(root)?;

    git::run(root, &["push", "origin", &branch])?;
    git::run(root, &["push", "origin", tag])?;

    Ok(())
}