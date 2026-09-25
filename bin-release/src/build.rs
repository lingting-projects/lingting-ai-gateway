use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::git;
use crate::metadata;
use crate::Platform;

const FRAMEWORK_REPOSITORY: &str =
    "https://github.com/lingting-projects/lingting-rust-framework.git";

const REACT_UI_REPOSITORY: &str =
    "https://github.com/lingting/lingting-react-ui.git";

const BINARY_NAME: &str = "bin-server";

pub fn run(platform: Platform, slim: bool) -> Result<()> {
    if matches!(platform, Platform::Macos) && slim {
        bail!("macOS does not support the slim build");
    }

    let root = git::repository_root()?;

    verify_environment()?;

    let info = metadata::ReleaseInfo::read(&metadata::info_path(&root))?;

    prepare_framework(&root, &info)?;
    prepare_react_ui(&root, &info)?;

    install_frontend_dependencies(&root)?;
    build_ts_sdk(&root)?;

    build_server(&root, platform, slim)?;
    rename_artifact(&root, platform, slim)?;

    Ok(())

}

fn verify_environment() -> Result<()> {
    for command in ["git", "cargo", "rustc", "node", "pnpm"] {
        require_command(command)?;
    }

    Ok(())

}

fn require_command(command: &str) -> Result<()> {
    let status = if cfg!(target_os = "windows") {
        Command::new("where")
            .arg(command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    } else {
        Command::new("sh")
            .args(["-c", &format!("command -v {command}")])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    }
        .with_context(|| format!("failed to check command {command}"))?;

    if !status.success() {
        bail!("required command is not available: {command}");
    }

    Ok(())

}

fn prepare_framework(
    root: &Path,
    info: &metadata::ReleaseInfo,
) -> Result<()> {
    let expected_repository = &info.framework.repository;

    if git::normalize_url(expected_repository)
        != git::normalize_url(FRAMEWORK_REPOSITORY)
    {
        bail!(
        "unsupported framework repository: {}",
        expected_repository
    );
    }

    let framework_dir = root
        .parent()
        .context("gateway repository has no parent directory")?
        .join("lingting-rust-framework");

    checkout_dependency(
        &framework_dir,
        expected_repository,
        &info.framework.commit,
    )

}

fn prepare_react_ui(
    root: &Path,
    info: &metadata::ReleaseInfo,
) -> Result<()> {
    if git::normalize_url(&info.react_ui.repository)
        != git::normalize_url(REACT_UI_REPOSITORY)
    {
        bail!(
"unsupported react-ui repository: {}",
info.react_ui.repository
);
    }

    let react_ui_dir = root
        .parent()
        .context("gateway repository has no parent directory")?
        .join("lingting-react-ui");

    checkout_dependency(
        &react_ui_dir,
        &info.react_ui.repository,
        &info.react_ui.commit,
    )?;

    prepare_lri(root, &react_ui_dir)

}

fn checkout_dependency(
    repository_dir: &Path,
    repository: &str,
    commit: &str,
) -> Result<()> {
    if !repository_dir.exists() {
        println!(
            "cloning {} into {}",
            repository,
            repository_dir.display()
        );

        git::clone(repository, repository_dir)?;
    } else {
        git::ensure_origin(repository_dir, repository)?;

        git::run(repository_dir, &["fetch", "--no-tags", "origin", commit])?;
    }

    git::fetch_commit(repository_dir, commit)?;
    git::checkout_detached(repository_dir, commit)?;
    git::ensure_commit(repository_dir, commit)?;

    Ok(())

}

fn prepare_lri(root: &Path, react_ui_dir: &Path) -> Result<()> {
    let ui_dir = root.join("lingting-ai-gateway-ui");
    let lri_dir = ui_dir.join("lri");
    let expected_src = react_ui_dir.join("src");

    if !ui_dir.is_dir() {
        bail!("gateway UI directory does not exist: {}", ui_dir.display());
    }

    if lri_dir.exists() {
        let metadata = fs::symlink_metadata(&lri_dir)
            .with_context(|| format!("failed to inspect {}", lri_dir.display()))?;

        if metadata.file_type().is_symlink() {
            let actual = fs::canonicalize(&lri_dir)
                .with_context(|| format!("failed to resolve {}", lri_dir.display()))?;

            let expected = fs::canonicalize(&expected_src)
                .with_context(|| format!("failed to resolve {}", expected_src.display()))?;

            if actual != expected {
                bail!(
                "lri points to an unexpected directory\nexpected: {}\nactual: {}",
                expected.display(),
                actual.display()
            );
            }

            return Ok(());
        }

        bail!(
        "lri exists but is not a symbolic link: {}",
        lri_dir.display()
    );
    }

    create_symlink(&expected_src, &lri_dir)

}

fn create_symlink(source: &Path, destination: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, destination).with_context(|| {
            format!(
                "failed to create symlink {} -> {}",
                destination.display(),
                source.display()
            )
        })?;
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, destination).with_context(
            || {
                format!(
                    "failed to create directory symlink {} -> {}",
                    destination.display(),
                    source.display()
                )
            },
        )?;
    }

    Ok(())

}

fn install_frontend_dependencies(root: &Path) -> Result<()> {
    let ui_dir = root.join("lingting-ai-gateway-ui");

    run_command(
        &ui_dir,
        "pnpm",
        &["install", "--frozen-lockfile"],
    )

}

fn build_ts_sdk(root: &Path) -> Result<()> {
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

fn build_server(
    root: &Path,
    platform: Platform,
    slim: bool,
) -> Result<()> {
    let target = match platform {
        Platform::Linux if slim => Some("x86_64-unknown-linux-gnu"),
        Platform::Linux => Some("x86_64-unknown-linux-musl"),
        Platform::Windows => Some("x86_64-pc-windows-msvc"),
        Platform::Macos => Some("x86_64-apple-darwin"),
    };

    let mut args = vec![
        "build",
        "--locked",
        "--package",
        BINARY_NAME,
        "--release",
    ];

    if let Some(target) = target {
        args.extend(["--target", target]);
    }

    let rustflags = match platform {
        Platform::Windows if slim => Some("-C target-feature=-crt-static"),
        Platform::Windows => Some("-C target-feature=+crt-static"),
        _ => None,
    };

    let mut command = Command::new("cargo");
    command.args(&args).current_dir(root);

    if let Some(rustflags) = rustflags {
        command.env("RUSTFLAGS", rustflags);
    }

    let status = command
        .status()
        .with_context(|| "failed to execute cargo build")?;

    if !status.success() {
        bail!("cargo build failed");
    }

    Ok(())

}

fn rename_artifact(
    root: &Path,
    platform: Platform,
    slim: bool,
) -> Result<()> {
    let target_dir = root.join("target");

    let (target, filename) = match platform {
        Platform::Linux if slim => (
            "x86_64-unknown-linux-gnu",
            "lingting-ai-gateway-linux-slim",
        ),
        Platform::Linux => (
            "x86_64-unknown-linux-musl",
            "lingting-ai-gateway-linux",
        ),
        Platform::Windows if slim => (
            "x86_64-pc-windows-msvc",
            "lingting-ai-gateway-windows-slim.exe",
        ),
        Platform::Windows => (
            "x86_64-pc-windows-msvc",
            "lingting-ai-gateway-windows.exe",
        ),
        Platform::Macos => (
            "x86_64-apple-darwin",
            "lingting-ai-gateway-macos",
        ),
    };

    let source = target_dir
        .join(target)
        .join("release")
        .join(if matches!(platform, Platform::Windows) {
            format!("{BINARY_NAME}.exe")
        } else {
            BINARY_NAME.to_owned()
        });

    if !source.is_file() {
        bail!("built binary does not exist: {}", source.display());
    }

    let dist_dir = root.join("dist");
    fs::create_dir_all(&dist_dir)
        .with_context(|| format!("failed to create {}", dist_dir.display()))?;

    let destination = dist_dir.join(filename);

    fs::copy(&source, &destination).with_context(|| {
        format!(
            "failed to copy {} to {}",
            source.display(),
            destination.display()
        )
    })?;

    println!("release artifact: {}", destination.display());

    Ok(())

}

fn run_command(root: &Path, program: &str, args: &[&str]) -> Result<()> {
    println!("running {program} {}", args.join(" "));

    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .with_context(|| {
            format!(
                "failed to execute `{program}` in {}",
                root.display()
            )
        })?;

    if !status.success() {
        bail!(
        "command failed: `{program} {}`",
        args.join(" ")
    );
    }

    Ok(())

}