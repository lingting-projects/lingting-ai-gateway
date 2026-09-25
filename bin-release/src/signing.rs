use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

const SIGN_IDENTITY: &str = "lingting-release";
const SIGN_NAMESPACE: &str = "lingting-release";

pub fn sign(info_file: &Path, signature_file: &Path, key_file: &Path) -> Result<()> {
    if !key_file.is_file() {
        bail!("signing key does not exist: {}", key_file.display());
    }

    let generated_file = info_file.with_extension("sig");

    if generated_file.exists() {
        fs::remove_file(&generated_file).with_context(|| {
            format!(
                "failed to remove existing signature: {}",
                generated_file.display()
            )
        })?;
    }

    let output = Command::new("ssh-keygen")
        .args([
            "-Y",
            "sign",
            "-f",
            &key_file.to_string_lossy(),
            "-n",
            SIGN_NAMESPACE,
            &info_file.to_string_lossy(),
        ])
        .output()
        .context("failed to execute ssh-keygen")?;

    if !output.status.success() {
        bail!(
            "failed to sign release metadata\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    if !generated_file.is_file() {
        bail!(
            "ssh-keygen did not create signature file: {}",
            generated_file.display()
        );
    }

    fs::rename(&generated_file, signature_file).with_context(|| {
        format!(
            "failed to move signature from {} to {}",
            generated_file.display(),
            signature_file.display()
        )
    })?;

    Ok(())
}

pub fn verify(
    info_file: &Path,
    signature_file: &Path,
    public_key_file: &Path,
) -> Result<()> {
    if !info_file.is_file() {
        bail!("release metadata does not exist: {}", info_file.display());
    }

    if !signature_file.is_file() {
        bail!(
            "release metadata signature does not exist: {}",
            signature_file.display()
        );
    }

    if !public_key_file.is_file() {
        bail!(
            "release public key does not exist: {}",
            public_key_file.display()
        );
    }

    let info = fs::File::open(info_file)
        .with_context(|| format!("failed to open {}", info_file.display()))?;

    let mut command = Command::new("ssh-keygen");

    command.args([
        "-Y",
        "verify",
        "-f",
        &public_key_file.to_string_lossy(),
        "-I",
        SIGN_IDENTITY,
        "-n",
        SIGN_NAMESPACE,
        "-s",
        &signature_file.to_string_lossy(),
    ]);

    let output = command
        .stdin(Stdio::from(info))
        .output()
        .context("failed to execute ssh-keygen")?;

    if !output.status.success() {
        bail!(
            "release metadata signature verification failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(())
}