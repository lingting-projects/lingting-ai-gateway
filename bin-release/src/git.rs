use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn run(root: &Path, args: &[&str]) -> Result<Output> {
    command(root, "git", args)
}

pub fn command(root: &Path, program: &str, args: &[&str]) -> Result<Output> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| {
            format!(
                "failed to execute {program} in {}",
                root.display()
            )
        })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        bail!(
        "command failed: `{program} {}`\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        stdout.trim(),
        stderr.trim()
    );
    }

    Ok(output)

}

pub fn output(root: &Path, args: &[&str]) -> Result<String> {
    let output = run(root, args)?;

    Ok(String::from_utf8(output.stdout)
        .context("git output is not valid UTF-8")?
        .trim()
        .to_owned())

}

pub fn repository_root() -> Result<PathBuf> {
    let current_dir =
        std::env::current_dir().context("failed to get current directory")?;

    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&current_dir)
        .output()
        .context("failed to execute git")?;

    if !output.status.success() {
        bail!(
        "current directory is not inside a Git repository: {}",
        current_dir.display()
    );
    }

    let root = String::from_utf8(output.stdout)
        .context("git repository path is not valid UTF-8")?
        .trim()
        .to_owned();

    Ok(PathBuf::from(root))

}

pub fn require_clean_tree(repo: &Path) -> Result<()> {
    let status = output(repo, &["status", "--porcelain"])?;

    if !status.is_empty() {
        bail!("Git worktree is not clean: {}", repo.display());
    }

    Ok(())

}

pub fn origin_url(repo: &Path) -> Result<String> {
    output(repo, &["remote", "get-url", "origin"])
        .with_context(|| format!("failed to get origin for {}", repo.display()))
}

pub fn current_branch(repo: &Path) -> Result<String> {
    output(repo, &["branch", "--show-current"])
        .with_context(|| format!("failed to get current branch for {}", repo.display()))
}

pub fn head_commit(repo: &Path) -> Result<String> {
    output(repo, &["rev-parse", "HEAD"])
        .with_context(|| format!("failed to get HEAD for {}", repo.display()))
}

pub fn remote_branch_commit(repo: &Path, branch: &str) -> Result<String> {
    output(
        repo,
        &["rev-parse", &format!("refs/remotes/origin/{branch}")],
    )
        .with_context(|| {
            format!(
                "failed to resolve origin/{branch} in {}",
                repo.display()
            )
        })
}

pub fn normalize_url(url: &str) -> String {
    let url = url.trim().trim_end_matches(".git");

    match url {
        url if url.starts_with("git@github.com:") => {
            url.trim_start_matches("git@github.com:").to_owned()
        }
        url if url.starts_with("ssh://git@github.com/") => {
            url.trim_start_matches("ssh://git@github.com/").to_owned()
        }
        url if url.starts_with("https://github.com/") => {
            url.trim_start_matches("https://github.com/").to_owned()
        }
        _ => url.to_owned(),
    }

}

pub fn ensure_origin(repo: &Path, expected: &str) -> Result<()> {
    let actual = origin_url(repo)?;

    if normalize_url(&actual) != normalize_url(expected) {
        bail!(
        "unexpected Git origin for {}\nexpected: {}\nactual: {}",
        repo.display(),
        expected,
        actual
    );
    }

    Ok(())

}

pub fn ensure_local_matches_remote(repo: &Path, branch: &str) -> Result<()> {
    let local = head_commit(repo)?;
    let remote = remote_branch_commit(repo, branch)?;

    if local != remote {
        bail!(
        "local HEAD does not match origin/{branch} in {}\nlocal: {}\nremote: {}",
        repo.display(),
        local,
        remote
    );
    }

    Ok(())

}

pub fn ensure_commit(repo: &Path, expected: &str) -> Result<()> {
    let actual = head_commit(repo)?;

    if actual != expected {
        bail!(
        "Git commit mismatch in {}\nexpected: {}\nactual: {}",
        repo.display(),
        expected,
        actual
    );
    }

    Ok(())

}

pub fn fetch_commit(repo: &Path, commit: &str) -> Result<()> {
    run(repo, &["fetch", "--no-tags", "origin", commit])?;
    Ok(())
}

pub fn checkout_detached(repo: &Path, commit: &str) -> Result<()> {
    run(repo, &["checkout", "--detach", commit])?;
    Ok(())
}

pub fn clone(repository: &str, destination: &Path) -> Result<()> {
    if destination.exists() {
        bail!(
"Git destination already exists: {}",
destination.display()
);
    }

    let destination = destination.to_string_lossy();

    Command::new("git")
        .args(["clone", "--no-tags", repository, &destination])
        .status()
        .with_context(|| format!("failed to clone {repository}"))?
        .success()
        .then_some(())
        .with_context(|| format!("failed to clone {repository}"))

}