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

    verify_dependency_repository(
        &framework_dir,
        FRAMEWORK_REPOSITORY,
        "framework",
    )?;

    verify_dependency_repository(
        &react_ui_dir,
        REACT_UI_REPOSITORY,
        "react-ui",
    )?;

    verify_cargo_metadata(&root)?;

    let version = cargo_version(&root)?;
    let tag = format!("v{version}");

    verify_tag_does_not_exist(&root, &tag)?;

    let framework = release_source(
        &framework_dir,
        FRAMEWORK_REPOSITORY,
        "framework",
    )?;

    let react_ui = release_source(
        &react_ui_dir,
        REACT_UI_REPOSITORY,
        "react-ui",
    )?;

    let info = ReleaseInfo::new(tag.clone(), framework, react_ui);

    let info_path = metadata::info_path(&root);
    let signature_path = metadata::signature_path(&root);
    let public_key_path = metadata::public_key_path(&root);

    info.write(&info_path)?;

    let sign_key = signing_key()?;

    signing::sign(&info_path, &signature_path, &sign_key)?;
    signing::verify(&info_path, &signature_path, &public_key_path)?;

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

    fetch_branch(root, &branch)?;
    git::ensure_local_matches_remote(root, &branch)?;

    Ok(())

}

fn verify_dependency_repository(
    repository: &Path,
    expected_origin: &str,
    name: &str,
) -> Result<()> {
    if !repository.is_dir() {
        bail!(
"{name} repository does not exist: {}",
repository.display()
);
    }

    git::ensure_origin(repository, expected_origin)?;
    git::require_clean_tree(repository)?;

    let branch = git::current_branch(repository)?;

    if branch.is_empty() {
        bail!("{name} repository is in detached HEAD state");
    }

    fetch_branch(repository, &branch)?;
    git::ensure_local_matches_remote(repository, &branch)?;

    Ok(())

}

fn release_source(
    repository: &Path,
    expected_origin: &str,
    name: &str,
) -> Result<ReleaseSource> {
    let branch = git::current_branch(repository)?;

    if branch.is_empty() {
        bail!("{name} repository is in detached HEAD state");
    }

    let commit = git::head_commit(repository)?;

    if commit.len() != 40 || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("invalid {name} commit: {commit}");
    }

    Ok(ReleaseSource {
        repository: expected_origin.to_owned(),
        branch,
        commit,
    })

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

fn fetch_branch(repository: &Path, branch: &str) -> Result<()> {
    git::run(
        repository,
        &["fetch", "--no-tags", "origin", branch],
    )?;
    Ok(())
}

fn verify_cargo_metadata(root: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .args(["metadata", "--locked", "--format-version", "1"])
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

    let mut in_workspace_package = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_workspace_package = line == "[workspace.package]";
            continue;
        }

        if in_workspace_package && line.starts_with("version") {
            if let Some((_, value)) = line.split_once('=') {
                let version = value.trim().trim_matches('"');

                if !version.is_empty() {
                    return Ok(version.to_owned());
                }
            }
        }
    }

    let mut in_package = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }

        if in_package && line.starts_with("version") {
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

fn verify_tag_does_not_exist(root: &Path, tag: &str) -> Result<()> {
    let local = Command::new("git")
        .args(["rev-parse", "-q", "--verify", &format!("refs/tags/{tag}")])
        .current_dir(root)
        .output()
        .context("failed to check local release tag")?;

    if local.status.success() {
        bail!("release tag already exists locally: {tag}");
    }

    let remote = Command::new("git")
        .args(["ls-remote", "--exit-code", "--tags", "origin", &format!("refs/tags/{tag}")])
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
        if !path.is_file() {
            bail!("required release file does not exist: {}", path.display());
        }
    }

    Ok(())

}

fn commit_metadata(root: &Path) -> Result<()> {
    git::run(
        root,
        &["add", ".release/info", ".release/info.sig"],
    )?;

    let staged = git::output(root, &["diff", "--cached", "--name-only"])?;

    if staged.trim().is_empty() {
        bail!("release metadata produced no Git changes");
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
        &["tag", "-a", tag, "-m", &format!("Release {tag}")],
    )?;

    Ok(())

}

fn push_release(root: &Path, tag: &str) -> Result<()> {
    let branch = git::current_branch(root)?;

    git::run(root, &["push", "origin", &branch])?;
    git::run(root, &["push", "origin", tag])?;

    Ok(())

}