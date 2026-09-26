use anyhow::{bail, Context, Result};
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

    let written = ReleaseInfo::read(&info_path)?;

    if written.render() != info.render() {
        bail!("release metadata changed unexpectedly after writing");
    }

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

    let src_name = real_lri
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    if src_name != "src" {
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

    let mut section = "";

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') && line.ends_with(']') {
            section = line;
            continue;
        }

        if section == "[workspace.package]" && line.starts_with("version") {
            if let Some((_, value)) = line.split_once('=') {
                let version = value.trim().trim_matches('"');

                if !version.is_empty() {
                    return Ok(version.to_owned());
                }
            }
        }
    }

    section = "";

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') && line.ends_with(']') {
            section = line;
            continue;
        }

        if section == "[package]" && line.starts_with("version") {
            if let Some((_, value)) = line.split_once('=') {
                let version = value.trim().trim_matches('"');

                if !version.is_empty() {
                    return Ok(version.to_owned());
                }
            }
        }
    }

    bail!("failed to find package version in {}", cargo_toml.display())
}

fn generate_framework_commands(
    root: &Path,
    framework: &ReleaseSource,
) -> Result<Vec<String>> {
    let cargo_toml = root.join("Cargo.toml");

    let content = fs::read_to_string(&cargo_toml)
        .with_context(|| format!("failed to read {}", cargo_toml.display()))?;

    let framework_dir = root
        .parent()
        .context("gateway repository has no parent directory")?
        .join("lingting-rust-framework");

    let framework_dir = fs::canonicalize(&framework_dir)
        .with_context(|| {
            format!(
                "failed to resolve framework repository: {}",
                framework_dir.display()
            )
        })?;

    let lines = content.lines().collect::<Vec<_>>();
    let mut commands = Vec::new();
    let mut packages = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];

        if !line.contains("path") || !line.contains("framework") {
            index += 1;
            continue;
        }

        let Some((dependency_name, start)) =
            dependency_assignment(&lines, index)
        else {
            index += 1;
            continue;
        };

        let mut end = start;
        let mut block = String::new();

        while end < lines.len() {
            if !block.is_empty() {
                block.push('\n');
            }

            block.push_str(lines[end]);

            if dependency_block_is_complete(&block) {
                break;
            }

            end += 1;
        }

        if end >= lines.len() {
            bail!(
                "unterminated dependency declaration for {}",
                dependency_name
            );
        }

        let Some(path_value) = extract_path_value(&block) else {
            index = end + 1;
            continue;
        };

        let resolved_path = if Path::new(&path_value).is_absolute() {
            PathBuf::from(&path_value)
        } else {
            cargo_toml
                .parent()
                .unwrap_or(root)
                .join(&path_value)
        };

        let resolved_path = fs::canonicalize(&resolved_path)
            .with_context(|| {
                format!(
                    "failed to resolve dependency path for {}: {}",
                    dependency_name,
                    resolved_path.display()
                )
            })?;

        if !resolved_path.starts_with(&framework_dir) {
            index = end + 1;
            continue;
        }

        commands.push(generate_toml_replace_command(
            &path_value,
            framework,
        ));

        packages.push(dependency_name);

        index = end + 1;
    }

    packages.sort();
    packages.dedup();

    if packages.is_empty() {
        bail!("no framework path dependencies were found in Cargo.toml");
    }

    commands.push(generate_cargo_update_command(&packages));

    Ok(commands)
}

fn dependency_assignment(
    lines: &[&str],
    index: usize,
) -> Option<(String, usize)> {
    let line = lines[index].trim();

    let (name, value) = line.split_once('=')?;

    let name = name.trim();

    if name.is_empty()
        || name.contains(' ')
        || name.contains('{')
        || name.contains('[')
    {
        return None;
    }

    if !value.contains('{') || !value.contains("path") {
        return None;
    }

    Some((name.to_owned(), index))
}

fn dependency_block_is_complete(block: &str) -> bool {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for byte in block.bytes() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }

            match byte {
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }

            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {}
        }
    }

    depth == 0
}

fn extract_path_value(block: &str) -> Option<String> {
    for line in block.lines() {
        let line = line.trim();

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        if key.trim() != "path" {
            continue;
        }

        let value = value
            .trim()
            .trim_end_matches(',')
            .trim()
            .trim_matches('"');

        if !value.is_empty() {
            return Some(value.to_owned());
        }
    }

    None
}

fn generate_toml_replace_command(
    path: &str,
    framework: &ReleaseSource,
) -> String {
    format!(
        "python3 - \"$ROOT_DIR/Cargo.toml\" \"{}\" \"{}\" \"{}\" \"{}\" <<'PY'\n\
import pathlib\n\
import sys\n\
\n\
cargo_toml = pathlib.Path(sys.argv[1])\n\
old_path = 'path = \"' + sys.argv[2] + '\"'\n\
new_dependency = 'git = \"' + sys.argv[3] + '\", branch = \"' + sys.argv[4] + '\", rev = \"' + sys.argv[5] + '\",'\n\
content = cargo_toml.read_text()\n\
if old_path not in content:\n\
    raise SystemExit('framework dependency path not found: ' + old_path)\n\
cargo_toml.write_text(content.replace(old_path, new_dependency))\n\
PY",
        shell_single_quote(path),
        shell_single_quote(framework.repository.as_str()),
        shell_single_quote(framework.branch.as_str()),
        shell_single_quote(framework.commit.as_str()),
    )
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

    let start_marker =
        "# === start\n# 这里用于 release 的 Rust 代码进行替换\n# === end";

    let start = original
        .find(start_marker)
        .context("release.sh is missing rsync_framework replacement markers")?;

    let function_start = original[start..]
        .find("rsync_framework()")
        .map(|offset| start + offset)
        .context("release.sh is missing rsync_framework function")?;

    let function_body_start = original[function_start..]
        .find('{')
        .map(|offset| function_start + offset + 1)
        .context("release.sh has an invalid rsync_framework function")?;

    let function_body_end =
        find_function_end(&original, function_body_start)
            .context("failed to locate rsync_framework function end")?;

    let mut function = String::from("rsync_framework() {\n");

    for command in framework_commands {
        for line in command.lines() {
            function.push_str("    ");
            function.push_str(line);
            function.push('\n');
        }
    }

    function.push('}');

    let mut updated =
        String::with_capacity(original.len() + function.len());

    updated.push_str(&original[..function_start]);
    updated.push_str(&function);
    updated.push_str(&original[function_body_end..]);

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

    while index < bytes.len() {
        match bytes[index] {
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

    let staged = git::output(
        root,
        &["diff", "--cached", "--name-only"],
    )?;

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