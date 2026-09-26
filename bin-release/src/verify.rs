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
    println!("[verify] ===== release verification started =====");

    let root = git::repository_root()?;
    println!("[verify] repository root: {}", root.display());

    println!("[verify] step 1/7: checking gateway repository");
    verify_gateway_repository(&root)?;
    println!("[verify] gateway repository verification passed");

    println!("[verify] step 2/7: checking release metadata files");
    verify_release_files(&root)?;
    println!("[verify] release metadata files verified");

    println!("[verify] step 3/7: reading release metadata");
    let info_path = metadata::info_path(&root);
    println!("[verify] release metadata path: {}", info_path.display());

    let info = ReleaseInfo::read(&info_path)?;

    println!("[verify] release tag: {}", info.tag);
    println!(
        "[verify] framework: repository={}, branch={}, commit={}",
        info.framework.repository, info.framework.branch, info.framework.commit
    );
    println!(
        "[verify] react-ui: repository={}, branch={}, commit={}",
        info.react_ui.repository, info.react_ui.branch, info.react_ui.commit
    );

    println!("[verify] step 4/7: verifying release tag");
    verify_tag(&root, &info)?;
    println!("[verify] release tag verification passed");

    println!("[verify] verifying framework metadata");
    verify_framework_metadata(&info)?;
    println!("[verify] framework metadata verification passed");

    println!("[verify] verifying react-ui metadata");
    verify_react_ui_metadata(&info)?;
    println!("[verify] react-ui metadata verification passed");

    println!("[verify] step 5/7: checking Cargo.lock and Cargo metadata");
    verify_cargo_metadata(&root)?;
    println!("[verify] Cargo metadata verification passed");

    println!("[verify] step 6/7: verifying release metadata signature");
    verify_signature(&root)?;
    println!("[verify] release metadata signature verification passed");

    println!("[verify] step 7/7: final verification");
    println!("[verify] ===== release verification passed =====");

    Ok(())
}

fn verify_gateway_repository(root: &Path) -> Result<()> {
    println!("[verify] checking gateway origin");
    git::ensure_origin(root, GATEWAY_REPOSITORY)?;

    println!("[verify] checking gateway working tree");
    git::require_clean_tree(root)?;

    /*
     * A GitHub Actions tag workflow checks out the tag itself. In that case
     * HEAD is intentionally detached, so detached HEAD must not be rejected
     * here. The exact tag is verified separately by verify_tag().
     */
    let branch = git::current_branch(root)?;

    if branch.is_empty() {
        println!(
            "[verify] gateway is in detached HEAD state; this is allowed for tag verification"
        );
    } else {
        println!("[verify] gateway branch: {branch}");
    }

    Ok(())
}

fn verify_release_files(root: &Path) -> Result<()> {
    let info = metadata::info_path(root);
    let signature = metadata::signature_path(root);
    let public_key = metadata::public_key_path(root);

    println!("[verify] release info: {}", info.display());
    println!("[verify] release signature: {}", signature.display());
    println!("[verify] release public key: {}", public_key.display());

    for path in [&info, &signature, &public_key] {
        if !path.is_file() {
            bail!("required release file does not exist: {}", path.display());
        }
    }

    let info_size = fs::metadata(&info)
        .with_context(|| format!("failed to stat {}", info.display()))?
        .len();

    println!("[verify] release metadata size: {info_size} bytes");

    if info_size == 0 {
        bail!("release metadata is empty: {}", info.display());
    }

    let signature_size = fs::metadata(&signature)
        .with_context(|| format!("failed to stat {}", signature.display()))?
        .len();

    println!("[verify] release signature size: {signature_size} bytes");

    if signature_size == 0 {
        bail!("release signature is empty: {}", signature.display());
    }

    let public_key_size = fs::metadata(&public_key)
        .with_context(|| format!("failed to stat {}", public_key.display()))?
        .len();

    println!("[verify] release public key size: {public_key_size} bytes");

    if public_key_size == 0 {
        bail!("release public key is empty: {}", public_key.display());
    }

    Ok(())
}

fn verify_tag(root: &Path, info: &ReleaseInfo) -> Result<()> {
    let expected_tag = &info.tag;

    println!("[verify] expected release tag: {expected_tag}");
    println!("[verify] resolving exact tag on HEAD");

    let actual_tag = git::output(root, &["describe", "--tags", "--exact-match", "HEAD"])?;

    let actual_tag = actual_tag.trim();

    println!("[verify] actual HEAD tag: {actual_tag}");

    if actual_tag != expected_tag {
        bail!(
            "HEAD tag mismatch\nexpected:\n  {}\nactual:\n  {}",
            expected_tag,
            actual_tag
        );
    }

    Ok(())
}

fn verify_framework_metadata(info: &ReleaseInfo) -> Result<()> {
    let repository = normalize_git_url(&info.framework.repository);
    let expected = normalize_git_url(FRAMEWORK_REPOSITORY);

    println!("[verify] normalized framework repository: {repository}");
    println!("[verify] expected framework repository: {expected}");

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

    println!("[verify] framework branch: {}", info.framework.branch);

    validate_sha1(&info.framework.commit, "framework commit")?;

    println!("[verify] framework commit format is valid");

    Ok(())
}

fn verify_react_ui_metadata(info: &ReleaseInfo) -> Result<()> {
    let repository = normalize_git_url(&info.react_ui.repository);
    let expected = normalize_git_url(REACT_UI_REPOSITORY);

    println!("[verify] normalized react-ui repository: {repository}");
    println!("[verify] expected react-ui repository: {expected}");

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

    println!("[verify] react-ui branch: {}", info.react_ui.branch);

    validate_sha1(&info.react_ui.commit, "react-ui commit")?;

    println!("[verify] react-ui commit format is valid");

    Ok(())
}

fn verify_cargo_metadata(root: &Path) -> Result<()> {
    println!(
        "[verify] executing: cargo metadata --locked --format-version 1 in {}",
        root.display()
    );

    let output = Command::new("cargo")
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(root)
        .output()
        .context("failed to execute cargo metadata")?;

    println!("[verify] cargo metadata exit status: {}", output.status);

    if !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("[verify] cargo metadata stdout:\n{}", stdout.trim());
    }

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("[verify] cargo metadata stderr:\n{}", stderr.trim());
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

fn verify_signature(root: &Path) -> Result<()> {
    let info = metadata::info_path(root);
    let signature = metadata::signature_path(root);
    let public_key = metadata::public_key_path(root);

    println!(
        "[verify] verifying signature: info={}, signature={}, public_key={}",
        info.display(),
        signature.display(),
        public_key.display()
    );

    signing::verify(&info, &signature, &public_key)
        .context("release metadata signature verification failed")
}

fn validate_sha1(value: &str, name: &str) -> Result<()> {
    println!("[verify] validating {name}");

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
