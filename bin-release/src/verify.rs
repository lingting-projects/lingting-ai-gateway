use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::git;
use crate::metadata::{self, ReleaseInfo};
use crate::signing;

const GATEWAY_REPOSITORY: &str = "https://github.com/lingting-projects/lingting-ai-gateway.git";

const FRAMEWORK_REPOSITORY: &str =
    "https://github.com/lingting-projects/lingting-rust-framework.git";

const REACT_UI_REPOSITORY: &str = "https://github.com/lingting/lingting-react-ui.git";

pub fn run() -> Result<()> {
    let root = git::repository_root()?;

    println!("==> Checking gateway repository");
    verify_gateway_repository(&root)?;

    println!("==> Checking release metadata files");
    verify_release_files(&root)?;

    println!("==> Checking release metadata");
    let info = ReleaseInfo::read(&metadata::info_path(&root))?;

    verify_tag(&root, &info)?;
    verify_framework_metadata(&info)?;
    verify_react_ui_metadata(&info)?;

    println!("==> Checking Cargo.lock");
    verify_cargo_metadata(&root)?;

    println!("==> Verifying release metadata signature");
    verify_signature(&root)?;

    println!("==> Release verification passed");

    Ok(())
}

fn verify_gateway_repository(root: &Path) -> Result<()> {
    git::ensure_origin(root, GATEWAY_REPOSITORY)?;
    git::require_clean_tree(root)?;

    let branch = git::current_branch(root)?;

    if branch.is_empty() {
        bail!("gateway repository is in detached HEAD state");
    }

    Ok(())
}

fn verify_release_files(root: &Path) -> Result<()> {
    let info = metadata::info_path(root);
    let signature = metadata::signature_path(root);
    let public_key = metadata::public_key_path(root);

    for path in [&info, &signature, &public_key] {
        if !path.is_file() {
            bail!("required release file does not exist: {}", path.display());
        }
    }

    if fs::metadata(&info)
        .with_context(|| format!("failed to stat {}", info.display()))?
        .len()
        == 0
    {
        bail!("release metadata is empty: {}", info.display());
    }

    if fs::metadata(&signature)
        .with_context(|| format!("failed to stat {}", signature.display()))?
        .len()
        == 0
    {
        bail!("release signature is empty: {}", signature.display());
    }

    if fs::metadata(&public_key)
        .with_context(|| format!("failed to stat {}", public_key.display()))?
        .len()
        == 0
    {
        bail!("release public key is empty: {}", public_key.display());
    }

    Ok(())
}

fn verify_tag(root: &Path, info: &ReleaseInfo) -> Result<()> {
    let expected_tag = &info.tag;

    let actual_tag = git::output(root, &["describe", "--tags", "--exact-match", "HEAD"])?;

    if actual_tag.trim() != expected_tag {
        bail!(
            "HEAD tag mismatch\nexpected:\n  {}\nactual:\n  {}",
            expected_tag,
            actual_tag.trim()
        );
    }

    Ok(())
}

fn verify_framework_metadata(info: &ReleaseInfo) -> Result<()> {
    let repository = normalize_git_url(&info.framework.repository);

    let expected = normalize_git_url(FRAMEWORK_REPOSITORY);

    if repository != expected {
        bail!(
            "framework repository mismatch\nexpected:\n  {}\nactual:\n  {}",
            FRAMEWORK_REPOSITORY,
            info.framework.repository
        );
    }

    if info.framework.branch.is_empty() {
        bail!("framework branch is empty");
    }

    validate_sha1(&info.framework.commit, "framework commit")?;

    Ok(())
}

fn verify_react_ui_metadata(info: &ReleaseInfo) -> Result<()> {
    let repository = normalize_git_url(&info.react_ui.repository);

    let expected = normalize_git_url(REACT_UI_REPOSITORY);

    if repository != expected {
        bail!(
            "react-ui repository mismatch\nexpected:\n  {}\nactual:\n  {}",
            REACT_UI_REPOSITORY,
            info.react_ui.repository
        );
    }

    if info.react_ui.branch.is_empty() {
        bail!("react-ui branch is empty");
    }

    validate_sha1(&info.react_ui.commit, "react-ui commit")?;

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

fn verify_signature(root: &Path) -> Result<()> {
    let info = metadata::info_path(root);
    let signature = metadata::signature_path(root);
    let public_key = metadata::public_key_path(root);

    signing::verify(&info, &signature, &public_key)
        .context("release metadata signature verification failed")
}

fn validate_sha1(value: &str, name: &str) -> Result<()> {
    if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("invalid {name}: {value}");
    }

    Ok(())
}

fn normalize_git_url(url: &str) -> String {
    let mut url = url.trim().trim_end_matches(".git");

    if let Some(value) = url.strip_prefix("git@github.com:") {
        return value.to_owned();
    }

    if let Some(value) = url.strip_prefix("ssh://git@github.com/") {
        return value.to_owned();
    }

    if let Some(value) = url.strip_prefix("https://github.com/") {
        return value.to_owned();
    }

    if let Some(value) = url.strip_prefix("http://github.com/") {
        return value.to_owned();
    }

    if let Some(value) = url.strip_prefix("https://www.github.com/") {
        return value.to_owned();
    }

    if let Some(value) = url.strip_prefix("http://www.github.com/") {
        return value.to_owned();
    }

    url = url.trim_end_matches('/');

    url.to_owned()
}
