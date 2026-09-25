use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

use crate::git;
use crate::metadata;
use crate::signing;

const GATEWAY_REPOSITORY: &str = "https://github.com/lingting-projects/lingting-ai-gateway.git";

const RELEASE_PUBKEY_FINGERPRINT: &str = "SHA256:+7HthGNGniZd5oo+s4+eDB7PzTATkKgofDRuWRtbgS4";

pub fn run() -> Result<()> {
    let root = git::repository_root()?;

    verify_release_directory(&root)?;
    verify_metadata(&root)?;
    verify_public_key(&root)?;
    verify_signature(&root)?;
    verify_gateway_repository(&root)?;

    println!("release verification succeeded");

    Ok(())
}

fn verify_release_directory(root: &Path) -> Result<()> {
    let release_dir = metadata::release_dir(root);

    if !release_dir.is_dir() {
        bail!(
            "release directory does not exist: {}",
            release_dir.display()
        );
    }

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

fn verify_metadata(root: &Path) -> Result<()> {
    let info_path = metadata::info_path(root);
    let info = metadata::ReleaseInfo::read(&info_path)?;

    let current_tag = git::output(root, &["describe", "--tags", "--exact-match", "HEAD"]).ok();

    if let Some(tag) = current_tag {
        if tag != info.tag {
            bail!(
                "release metadata tag does not match current tag\nmetadata: {}\ncurrent: {}",
                info.tag,
                tag
            );
        }
    }

    Ok(())
}

fn verify_public_key(root: &Path) -> Result<()> {
    let public_key = metadata::public_key_path(root);

    let output = std::process::Command::new("ssh-keygen")
        .args(["-lf", &public_key.to_string_lossy(), "-E", "sha256"])
        .output()
        .context("failed to execute ssh-keygen")?;

    if !output.status.success() {
        bail!(
            "failed to inspect release public key\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    if !stdout.contains(RELEASE_PUBKEY_FINGERPRINT) {
        bail!(
            "release public key fingerprint mismatch\nexpected: {}\nactual:\n{}",
            RELEASE_PUBKEY_FINGERPRINT,
            stdout.trim()
        );
    }

    Ok(())
}

fn verify_signature(root: &Path) -> Result<()> {
    signing::verify(
        &metadata::info_path(root),
        &metadata::signature_path(root),
        &metadata::public_key_path(root),
    )
}

fn verify_gateway_repository(root: &Path) -> Result<()> {
    git::ensure_origin(root, GATEWAY_REPOSITORY)?;
    git::require_clean_tree(root)?;

    let cargo_toml = root.join("Cargo.toml");

    if !cargo_toml.is_file() {
        bail!("Cargo.toml does not exist: {}", cargo_toml.display());
    }

    let cargo_metadata = std::process::Command::new("cargo")
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(root)
        .output()
        .context("failed to execute cargo metadata")?;

    if !cargo_metadata.status.success() {
        bail!(
            "cargo metadata verification failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&cargo_metadata.stdout).trim(),
            String::from_utf8_lossy(&cargo_metadata.stderr).trim()
        );
    }

    verify_no_crlf(&metadata::info_path(root))?;
    verify_no_crlf(&metadata::signature_path(root))?;
    verify_no_crlf(&metadata::public_key_path(root))?;

    Ok(())
}

fn verify_no_crlf(path: &Path) -> Result<()> {
    let content = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;

    if content.windows(2).any(|window| window == b"\r\n") {
        bail!(
            "release file contains CRLF line endings: {}",
            path.display()
        );
    }

    if content.contains(&b'\r') {
        bail!("release file contains CR characters: {}", path.display());
    }

    Ok(())
}
