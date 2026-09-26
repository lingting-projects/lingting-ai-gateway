use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::git;
use crate::metadata::{self, ReleaseInfo, ReleaseSource};
use crate::signing;

const GATEWAY_REPOSITORY: &str = "https://github.com/lingting-projects/lingting-ai-gateway.git";

const FRAMEWORK_REPOSITORY: &str =
    "https://github.com/lingting-projects/lingting-rust-framework.git";

const REACT_UI_REPOSITORY: &str = "https://github.com/lingting/lingting-react-ui.git";

const SIGN_KEY_ENV: &str = "LINGTING_RELEASE_SIGN_KEY";

pub fn run() -> Result<()> {
    println!("[release] ===== release tag started =====");

    let root = git::repository_root()?;
    println!("[release] repository root: {}", root.display());

    println!("[release] step 1/10: verifying gateway repository");
    verify_gateway(&root)?;
    println!("[release] gateway repository verification passed");

    let framework_dir = root
        .parent()
        .context("gateway repository has no parent directory")?
        .join("lingting-rust-framework");

    println!(
        "[release] framework repository path: {}",
        framework_dir.display()
    );

    println!("[release] resolving react-ui repository");
    let react_ui_dir = resolve_react_ui_repository(&root)?;
    println!(
        "[release] react-ui repository path: {}",
        react_ui_dir.display()
    );

    println!("[release] step 2/10: verifying framework repository");
    let framework = verify_framework_repository(&framework_dir)?;
    println!(
        "[release] framework verified: branch={}, commit={}",
        framework.branch, framework.commit
    );

    println!("[release] step 3/10: verifying react-ui repository");
    let react_ui = verify_react_ui_repository(&react_ui_dir)?;
    println!(
        "[release] react-ui verified: branch={}, commit={}",
        react_ui.branch, react_ui.commit
    );

    println!("[release] step 4/10: verifying cargo metadata");
    verify_cargo_metadata(&root)?;
    println!("[release] cargo metadata verification passed");

    println!("[release] resolving gateway version");
    let version = cargo_version(&root)?;
    let tag = format!("v{version}");
    println!("[release] gateway version: {version}");
    println!("[release] release tag: {tag}");

    println!("[release] step 5/10: checking release tag");
    verify_tag_does_not_exist(&root, &tag)?;
    println!("[release] release tag is available");

    println!("[release] step 6/10: generating framework dependency commands");
    let framework_commands = generate_framework_commands(&root, &framework)?;

    println!(
        "[release] generated {} framework release commands",
        framework_commands.len()
    );

    for (index, command) in framework_commands.iter().enumerate() {
        println!("[release] generated command {}: {}", index + 1, command);
    }

    println!("[release] step 7/10: updating release script");
    update_release_script(&root, &framework_commands)?;
    println!("[release] release script updated");

    println!("[release] generating release metadata");
    let info = ReleaseInfo::new(tag.clone(), framework, react_ui);

    let info_path = metadata::info_path(&root);
    let signature_path = metadata::signature_path(&root);
    let public_key_path = metadata::public_key_path(&root);

    println!(
        "[release] writing release metadata: {}",
        info_path.display()
    );
    info.write(&info_path)?;

    println!("[release] resolving release signing key");
    let sign_key = signing_key()?;
    println!("[release] signing key: {}", sign_key.display());

    println!("[release] signing release metadata");
    signing::sign(&info_path, &signature_path, &sign_key)?;

    println!("[release] verifying release metadata signature");
    signing::verify(&info_path, &signature_path, &public_key_path)?;

    println!("[release] step 8/10: verifying generated release files");
    verify_release_files(&root)?;
    println!("[release] generated release files verified");

    println!("[release] step 9/10: committing release metadata");
    commit_metadata(&root)?;
    println!("[release] release metadata committed");

    println!("[release] creating annotated release tag");
    create_tag(&root, &tag)?;
    println!("[release] annotated release tag created");

    println!("[release] step 10/10: pushing release");
    push_release(&root, &tag)?;
    println!("[release] release branch and tag pushed");

    println!("[release] ===== release tag completed: {tag} =====");

    Ok(())
}

fn verify_gateway(root: &Path) -> Result<()> {
    println!("[release] checking gateway origin");
    git::ensure_origin(root, GATEWAY_REPOSITORY)?;

    println!("[release] checking gateway working tree");
    git::require_clean_tree(root)?;

    println!("[release] checking gateway branch");
    let branch = git::current_branch(root)?;

    println!("[release] gateway branch: {branch}");

    if branch.is_empty() {
        bail!("gateway is in detached HEAD state");
    }

    Ok(())
}

fn verify_framework_repository(repository: &Path) -> Result<ReleaseSource> {
    if !repository.is_dir() {
        bail!(
            "framework repository does not exist: {}",
            repository.display()
        );
    }

    println!("[release] checking framework origin");
    git::ensure_origin(repository, FRAMEWORK_REPOSITORY)?;

    println!("[release] checking framework working tree");
    git::require_clean_tree(repository)?;

    println!("[release] checking framework branch");
    let branch = git::current_branch(repository)?;

    println!("[release] framework branch: {branch}");

    if branch.is_empty() {
        bail!("framework repository is in detached HEAD state");
    }

    println!("[release] reading framework HEAD commit");
    let commit = git::head_commit(repository)?;

    println!("[release] framework HEAD: {commit}");

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

    println!("[release] checking react-ui origin");
    git::ensure_origin(repository, REACT_UI_REPOSITORY)?;

    println!("[release] checking react-ui working tree");
    git::require_clean_tree(repository)?;

    println!("[release] checking react-ui branch");
    let branch = git::current_branch(repository)?;

    println!("[release] react-ui branch: {branch}");

    if branch.is_empty() {
        bail!("react-ui repository is in detached HEAD state");
    }

    println!("[release] reading react-ui HEAD commit");
    let commit = git::head_commit(repository)?;

    println!("[release] react-ui HEAD: {commit}");

    validate_commit(&commit, "react-ui commit")?;

    Ok(ReleaseSource {
        repository: REACT_UI_REPOSITORY.to_owned(),
        branch,
        commit,
    })
}

fn validate_commit(commit: &str, name: &str) -> Result<()> {
    println!("[release] validating {name}: {commit}");

    if commit.len() != 40 || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("invalid {name}: {commit}");
    }

    Ok(())
}

fn resolve_react_ui_repository(root: &Path) -> Result<PathBuf> {
    let lri = root.join("lingting-ai-gateway-ui").join("lri");

    println!("[release] react-ui link path: {}", lri.display());

    if !lri.exists() {
        bail!("lri does not exist: {}", lri.display());
    }

    let real_lri =
        fs::canonicalize(&lri).with_context(|| format!("failed to resolve {}", lri.display()))?;

    println!("[release] resolved lri path: {}", real_lri.display());

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

    println!(
        "[release] resolved react-ui repository: {}",
        repository.display()
    );

    if !repository.join(".git").exists() {
        bail!(
            "resolved react-ui directory is not a Git repository: {}",
            repository.display()
        );
    }

    Ok(repository)
}

fn verify_cargo_metadata(root: &Path) -> Result<()> {
    println!(
        "[release] executing: cargo metadata --locked --format-version 1 in {}",
        root.display()
    );

    let output = Command::new("cargo")
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(root)
        .output()
        .context("failed to execute cargo metadata")?;

    println!("[release] cargo metadata exit status: {}", output.status);

    if !output.stdout.is_empty() {
        println!(
            "[release] cargo metadata stdout:\n{}",
            String::from_utf8_lossy(&output.stdout).trim()
        );
    }

    if !output.stderr.is_empty() {
        println!(
            "[release] cargo metadata stderr:\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

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

    println!("[release] reading Cargo.toml: {}", cargo_toml.display());

    let content = fs::read_to_string(&cargo_toml)
        .with_context(|| format!("failed to read {}", cargo_toml.display()))?;

    if let Some(version) = find_section_version(&content, "[workspace.package]") {
        println!("[release] version found in [workspace.package]: {version}");
        return Ok(version);
    }

    if let Some(version) = find_section_version(&content, "[package]") {
        println!("[release] version found in [package]: {version}");
        return Ok(version);
    }

    bail!("failed to find version in {}", cargo_toml.display())
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

fn generate_framework_commands(root: &Path, framework: &ReleaseSource) -> Result<Vec<String>> {
    let cargo_toml = root.join("Cargo.toml");

    println!(
        "[release] scanning framework dependencies in {}",
        cargo_toml.display()
    );

    let content = fs::read_to_string(&cargo_toml)
        .with_context(|| format!("failed to read {}", cargo_toml.display()))?;

    let mut in_workspace_dependencies = false;
    let mut framework_dependencies = Vec::new();

    for (line_number, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_dependencies = trimmed == "[workspace.dependencies]";

            if in_workspace_dependencies {
                println!(
                    "[release] entered [workspace.dependencies] at line {}",
                    line_number + 1
                );
            }

            continue;
        }

        if !in_workspace_dependencies || !trimmed.starts_with("framework-") {
            continue;
        }

        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };

        let name = name.trim();

        if !name.starts_with("framework-") {
            continue;
        }

        println!(
            "[release] found framework dependency: line={}, name={}, original={}",
            line_number + 1,
            name,
            line
        );

        framework_dependencies.push((line_number + 1, line.to_owned(), name.to_owned()));
    }

    if framework_dependencies.is_empty() {
        bail!(
            "no framework-* dependencies found in [workspace.dependencies] of {}",
            cargo_toml.display()
        );
    }

    println!(
        "[release] framework dependency count: {}",
        framework_dependencies.len()
    );

    let mut commands = Vec::with_capacity(framework_dependencies.len() + 1);

    for (line_number, original_line, name) in &framework_dependencies {
        let replacement = build_git_dependency_line(original_line, framework);

        println!(
            "[release] replacement for {} at line {}: {}",
            name, line_number, replacement
        );

        commands.push(generate_sed_replace_command(*line_number, &replacement));
    }

    let mut packages = framework_dependencies
        .into_iter()
        .map(|(_, _, name)| name)
        .collect::<Vec<_>>();

    packages.sort();
    packages.dedup();

    println!(
        "[release] framework packages selected for cargo update: {}",
        packages.join(", ")
    );

    commands.push(generate_cargo_update_command(&packages));

    Ok(commands)
}

fn generate_sed_replace_command(line_number: usize, replacement: &str) -> String {
    let replacement = sed_escape_replacement(replacement);

    format!(
        "sed -i '{}c\\{}' \"$ROOT_DIR/Cargo.toml\"",
        line_number, replacement
    )
}

fn sed_escape_replacement(value: &str) -> String {
    value.replace('\\', r"\\").replace('&', r"\&")
}

fn build_git_dependency_line(original_line: &str, framework: &ReleaseSource) -> String {
    let leading_len =
        original_line.len() - original_line.trim_start_matches(char::is_whitespace).len();

    let leading = &original_line[..leading_len];
    let trimmed = original_line.trim();

    let Some((name, value)) = trimmed.split_once('=') else {
        return original_line.to_owned();
    };

    let name = name.trim();
    let value = value.trim();

    let features = extract_inline_attribute(value, "features");
    let default_features = extract_inline_attribute(value, "default-features");

    let mut dependency = format!(
        "{} = {{ git = \"{}\", rev = \"{}\"",
        name, framework.repository, framework.commit,
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

fn extract_inline_attribute(value: &str, attribute: &str) -> Option<String> {
    let marker = attribute;
    let mut search_start = 0usize;

    while let Some(relative_start) = value[search_start..].find(marker) {
        let start = search_start + relative_start;
        let after_name = &value[start + marker.len()..];

        if !after_name
            .chars()
            .next()
            .is_some_and(|character| character.is_whitespace() || character == '=')
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
                    return Some(after_equals[..offset].trim().to_owned());
                }
                _ => {}
            }
        }

        return Some(after_equals.trim().trim_end_matches('}').trim().to_owned());
    }

    None
}

fn generate_cargo_update_command(packages: &[String]) -> String {
    /*
     * Do not use `cargo update -p framework-core ...` here.
     *
     * `cargo update -p` operates on an existing package ID in Cargo.lock.
     * After replacing a path dependency with a git dependency, the package
     * identity/source can change and Cargo may report:
     *
     *   package ID specification `framework-core` did not match any packages
     *
     * Let Cargo resolve the changed framework git dependencies from the
     * updated Cargo.toml instead. The generated command still appears once
     * after all framework dependency replacements.
     */
    let _ = packages;

    "cargo update".to_owned()
}

fn shell_word(value: &str) -> String {
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'/'))
    {
        value.to_owned()
    } else {
        shell_single_quote(value)
    }
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn update_release_script(root: &Path, framework_commands: &[String]) -> Result<()> {
    let release_script = metadata::release_dir(root).join("release.sh");

    println!(
        "[release] reading release script: {}",
        release_script.display()
    );

    let original = fs::read_to_string(&release_script)
        .with_context(|| format!("failed to read {}", release_script.display()))?;

    let function_start = original
        .find("rsync_framework()")
        .context("release.sh is missing rsync_framework function")?;

    let body_start = original[function_start..]
        .find('{')
        .map(|offset| function_start + offset + 1)
        .context("release.sh has an invalid rsync_framework function")?;

    let body_end = find_function_end(&original, body_start)
        .context("failed to locate rsync_framework function end")?;

    println!(
        "[release] replacing rsync_framework body: {}..{}",
        function_start, body_end
    );

    let mut function = String::from("rsync_framework() {\n");

    for command in framework_commands {
        for line in command.lines() {
            let line = line.trim_end();

            if line.is_empty() {
                continue;
            }

            function.push_str("    ");
            function.push_str(line);
            function.push('\n');
        }
    }

    function.push('}');

    let mut updated = String::with_capacity(original.len() + function.len());

    updated.push_str(&original[..function_start]);
    updated.push_str(&function);
    updated.push_str(&original[body_end..]);

    let updated = updated
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");

    let updated = format!("{updated}\n");

    fs::write(&release_script, updated)
        .with_context(|| format!("failed to write {}", release_script.display()))?;

    println!(
        "[release] release script written: {}",
        release_script.display()
    );

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
    println!("[release] checking local tag: {tag}");

    let local = Command::new("git")
        .args(["rev-parse", "-q", "--verify", &format!("refs/tags/{tag}")])
        .current_dir(root)
        .output()
        .context("failed to check local release tag")?;

    if local.status.success() {
        bail!("release tag already exists locally: {tag}");
    }

    println!("[release] checking remote tag: {tag}");

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

        println!(
            "[release] signing key requested through {}: {}",
            SIGN_KEY_ENV,
            path.display()
        );

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
    let path = home.join(".ssh").join("lingting_gateway_ed25519");

    println!(
        "[release] using default signing key path: {}",
        path.display()
    );

    if !path.is_file() {
        bail!("release signing key does not exist: {}", path.display());
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
        println!("[release] checking release file: {}", path.display());

        if !path.is_file() {
            bail!("required release file does not exist: {}", path.display());
        }
    }

    Ok(())
}

fn commit_metadata(root: &Path) -> Result<()> {
    println!("[release] staging release metadata files");

    git::run(
        root,
        &[
            "add",
            ".release/release.sh",
            ".release/info",
            ".release/info.sig",
        ],
    )?;

    println!("[release] validating staged diff");

    git::run(root, &["diff", "--cached", "--check"])?;

    let staged = git::output(root, &["diff", "--cached", "--name-only"])?;

    println!("[release] staged files:\n{}", staged.trim());

    if staged.trim().is_empty() {
        bail!("release produced no Git changes");
    }

    println!("[release] creating release metadata commit");

    git::run(
        root,
        &["commit", "-m", "chore(release): update release metadata"],
    )?;

    Ok(())
}

fn create_tag(root: &Path, tag: &str) -> Result<()> {
    println!("[release] creating annotated tag: {tag}");

    git::run(root, &["tag", "-a", tag, "-m", &format!("Release {tag}")])?;

    Ok(())
}

fn push_release(root: &Path, tag: &str) -> Result<()> {
    let branch = git::current_branch(root)?;

    println!("[release] pushing branch: {branch}");
    git::run(root, &["push", "origin", &branch])?;

    println!("[release] pushing tag: {tag}");
    git::run(root, &["push", "origin", tag])?;

    Ok(())
}
