use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::Platform;
use crate::metadata::{self, ReleaseInfo};

pub fn run(platform: Platform, slim: bool) -> Result<()> {
    let root = crate::git::repository_root()?;

    let info = ReleaseInfo::read(&metadata::info_path(&root))?;

    prepare_react_ui(&root, &info)?;

    build_ts_sdk(&root)?;
    build_server(&root, platform, slim)?;

    Ok(())
}

fn prepare_react_ui(root: &Path, info: &ReleaseInfo) -> Result<()> {
    let ui_root = root.join("lingting-ai-gateway-ui");
    let lri = ui_root.join("lri");

    let repository = root
        .parent()
        .context("gateway repository has no parent directory")?
        .join("lingting-react-ui");

    if !repository.is_dir() {
        bail!(
            "react-ui repository does not exist: {}",
            repository.display()
        );
    }

    let current_origin = crate::git::origin_url(&repository)?;

    if crate::git::normalize_url(&current_origin)
        != crate::git::normalize_url(&info.react_ui.repository)
    {
        bail!(
            "react-ui repository origin mismatch\nexpected: {}\nactual: {}",
            info.react_ui.repository,
            current_origin
        );
    }

    crate::git::require_clean_tree(&repository)?;

    let current_branch = crate::git::current_branch(&repository)?;

    if current_branch.is_empty() {
        bail!("react-ui repository is in detached HEAD state");
    }

    crate::git::ensure_commit(&repository, &info.react_ui.commit)?;

    if lri.exists() {
        let metadata = fs::symlink_metadata(&lri)
            .with_context(|| format!("failed to stat {}", lri.display()))?;

        if metadata.file_type().is_symlink() {
            fs::remove_file(&lri).with_context(|| format!("failed to remove {}", lri.display()))?;
        } else if metadata.is_dir() {
            fs::remove_dir_all(&lri)
                .with_context(|| format!("failed to remove {}", lri.display()))?;
        } else {
            fs::remove_file(&lri).with_context(|| format!("failed to remove {}", lri.display()))?;
        }
    }

    fs::create_dir_all(&ui_root)
        .with_context(|| format!("failed to create {}", ui_root.display()))?;

    create_symlink(&repository.join("src"), &lri)?;

    install_frontend_dependencies(&ui_root)?;

    Ok(())
}

fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).with_context(|| {
            format!(
                "failed to create symlink {} -> {}",
                link.display(),
                target.display()
            )
        })?;
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(target, link).with_context(|| {
            format!(
                "failed to create symlink {} -> {}",
                link.display(),
                target.display()
            )
        })?;
    }

    Ok(())
}

fn install_frontend_dependencies(ui_root: &Path) -> Result<()> {
    require_command("pnpm")?;

    run_command(ui_root, "pnpm", &["install", "--frozen-lockfile"])
}

fn build_ts_sdk(root: &Path) -> Result<()> {
    println!("==> Building TypeScript SDK");

    run_command(
        root,
        "cargo",
        &[
            "run",
            "--locked",
            "--package",
            "lib-web",
            "--example",
            "build_ts",
            "--features",
            "ts-export",
        ],
    )
}

fn build_server(root: &Path, platform: Platform, slim: bool) -> Result<()> {
    let (target, output_name, rustflags) = build_target(platform, slim)?;

    println!("==> Building server: target={}, slim={}", target, slim);

    let mut command = Command::new("cargo");

    command.current_dir(root);
    command.args(["build", "--locked", "--release", "--target", target]);

    if let Some(rustflags) = rustflags {
        command.env("RUSTFLAGS", rustflags);
    }

    let status = command
        .status()
        .with_context(|| format!("failed to execute cargo build for target {target}"))?;

    if !status.success() {
        bail!("cargo build failed for target {target}");
    }

    let source = root
        .join("target")
        .join(target)
        .join("release")
        .join(if cfg!(windows) {
            "lingting-ai-gateway.exe"
        } else {
            "lingting-ai-gateway"
        });

    if !source.is_file() {
        bail!("built server artifact does not exist: {}", source.display());
    }

    let dist = root.join("dist");

    fs::create_dir_all(&dist).with_context(|| format!("failed to create {}", dist.display()))?;

    let destination = dist.join(output_name);

    fs::copy(&source, &destination).with_context(|| {
        format!(
            "failed to copy {} to {}",
            source.display(),
            destination.display()
        )
    })?;

    println!("==> Artifact: {}", destination.display());

    Ok(())
}

fn build_target(
    platform: Platform,
    slim: bool,
) -> Result<(&'static str, &'static str, Option<&'static str>)> {
    match platform {
        Platform::Linux if slim => Ok((
            "x86_64-unknown-linux-gnu",
            "lingting-ai-gateway-linux-slim",
            None,
        )),

        Platform::Linux => Ok((
            "x86_64-unknown-linux-musl",
            "lingting-ai-gateway-linux",
            None,
        )),

        Platform::Windows if slim => Ok((
            "x86_64-pc-windows-msvc",
            "lingting-ai-gateway-windows-slim.exe",
            Some("-C target-feature=-crt-static"),
        )),

        Platform::Windows => Ok((
            "x86_64-pc-windows-msvc",
            "lingting-ai-gateway-windows.exe",
            Some("-C target-feature=+crt-static"),
        )),

        Platform::Macos if slim => {
            bail!("macOS does not support the slim build")
        }

        Platform::Macos => Ok(("x86_64-apple-darwin", "lingting-ai-gateway-macos", None)),
    }
}

fn require_command(command: &str) -> Result<()> {
    let status = Command::new(command)
        .arg("--version")
        .status()
        .with_context(|| format!("failed to execute `{command}`"))?;

    if !status.success() {
        bail!("required command is unavailable: {command}");
    }

    Ok(())
}

fn run_command(root: &Path, program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .with_context(|| format!("failed to execute `{program} {}`", args.join(" ")))?;

    if !status.success() {
        bail!("command failed: `{program} {}`", args.join(" "));
    }

    Ok(())
}
