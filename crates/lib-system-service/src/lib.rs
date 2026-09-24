//! 系统服务注册与卸载。
//!
//! 对外只暴露统一配置 [`ServiceConfig`] 与统一入口 [`ServiceManager`]，
//! 各操作系统的差异由本 crate 内部承担：
//!
//! - Windows：任务计划程序（`schtasks`）
//! - Linux：systemd（`/etc/systemd/system` + `systemctl`）
//! - macOS：launchd（`/Library/LaunchDaemons` + `launchctl`）
//!
//! 各平台的实现按编译目标门控，非目标平台的代码与依赖不参与编译。

pub mod config;
pub mod manager;

mod command;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod file;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

pub use config::ServiceConfig;
pub use manager::ServiceManager;
