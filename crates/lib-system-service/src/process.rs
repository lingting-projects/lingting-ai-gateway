//! 查找并终止与指定可执行文件相同的进程。

use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use sysinfo::{Pid, System};

/// 等待进程退出的超时时间。
const EXIT_TIMEOUT: Duration = Duration::from_secs(10);

/// 轮询进程是否退出的间隔。
const EXIT_INTERVAL: Duration = Duration::from_millis(100);

/// 终止所有正在运行同一可执行文件的进程，不含当前进程，并等待其退出。
///
/// 找不到目标进程或终止失败都直接报错，避免调用方在旧进程仍在运行时继续启动新进程。
pub fn kill_same_executable(program: &Path) -> Result<Vec<u32>> {
    let current = std::process::id();
    let system = System::new_all();
    let mut killed = Vec::new();
    let mut failed = Vec::new();

    for (pid, process) in system.processes() {
        let pid = pid.as_u32();
        // 当前进程自身也运行同一个可执行文件，必须排除。
        if pid == current || !is_same_executable(process.exe(), program) {
            continue;
        }

        if process.kill() {
            killed.push(pid);
        } else {
            failed.push(pid);
        }
    }

    if !failed.is_empty() {
        bail!("终止进程失败，可能需要更高权限：{failed:?}");
    }
    if killed.is_empty() {
        bail!("未找到正在运行的进程：{}", program.display());
    }

    wait_exit(&killed)?;

    Ok(killed)
}

/// 判断进程的可执行文件是否与目标一致；路径无法归一化时退化为直接比较。
fn is_same_executable(candidate: Option<&Path>, program: &Path) -> bool {
    let Some(candidate) = candidate.filter(|path| !path.as_os_str().is_empty()) else {
        return false;
    };

    match (normalize(candidate), normalize(program)) {
        (Some(candidate), Some(program)) => candidate == program,
        _ => candidate == program,
    }
}

/// 归一化路径，用于消除相对路径、符号链接与大小写差异。
fn normalize(path: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(path).ok()
}

/// 等待被终止的进程退出，避免端口尚未释放就启动新进程。
fn wait_exit(pids: &[u32]) -> Result<()> {
    let deadline = Instant::now() + EXIT_TIMEOUT;

    loop {
        let system = System::new_all();
        let alive = pids
            .iter()
            .filter(|pid| system.process(Pid::from_u32(**pid)).is_some())
            .collect::<Vec<_>>();
        if alive.is_empty() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!("等待进程退出超时：{alive:?}");
        }

        sleep(EXIT_INTERVAL);
    }
}
