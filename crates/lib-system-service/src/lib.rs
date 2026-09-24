//! 系统服务注册与卸载。
//!
//! 对外只暴露统一配置 [`ServiceConfig`] 与统一入口 [`ServiceManager`]，
//! 各操作系统的差异由本 crate 内部承担：
//!
//! - Windows：任务计划程序（`schtasks`）
//! - Linux：systemd（`/etc/systemd/system` + `systemctl`）
//! - macOS：launchd（`/Library/LaunchDaemons` + `launchctl`）
//!
//! 三个平台的实现始终参与编译，避免非本机平台的代码长期失检。

pub mod config;
pub mod manager;

mod command;
mod file;
mod linux;
mod macos;
mod windows;

pub use config::ServiceConfig;
pub use manager::ServiceManager;
