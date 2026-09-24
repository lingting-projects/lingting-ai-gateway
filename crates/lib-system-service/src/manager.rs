use anyhow::Result;

use crate::config::ServiceConfig;
/// 系统服务管理入口；按当前操作系统选择实现。
pub struct ServiceManager {
    platform: Box<dyn Platform>,
}

impl ServiceManager {
    /// 创建当前操作系统的服务管理入口。
    pub fn new() -> Result<Self> {
        Ok(Self {
            platform: current_platform()?,
        })
    }

    /// 注册服务；`autostart` 为真时随系统启动。
    pub fn install(&self, config: &ServiceConfig) -> Result<()> {
        self.platform.install(config)
    }

    /// 注销服务；不停止正在运行的进程，需要先停止时由调用方先调 [`ServiceManager::stop`]。
    pub fn uninstall(&self, config: &ServiceConfig) -> Result<()> {
        self.platform.uninstall(config)
    }

    /// 启动服务。
    pub fn start(&self, config: &ServiceConfig) -> Result<()> {
        self.platform.start(config)
    }

    /// 停止服务。
    pub fn stop(&self, config: &ServiceConfig) -> Result<()> {
        self.platform.stop(config)
    }
}

/// 各操作系统的服务实现。
pub(crate) trait Platform {
    /// 注册服务。
    fn install(&self, config: &ServiceConfig) -> Result<()>;

    /// 注销服务。
    fn uninstall(&self, config: &ServiceConfig) -> Result<()>;

    /// 启动服务。
    fn start(&self, config: &ServiceConfig) -> Result<()>;

    /// 停止服务。
    fn stop(&self, config: &ServiceConfig) -> Result<()>;
}

/// 当前操作系统的实现。
#[cfg(target_os = "windows")]
fn current_platform() -> Result<Box<dyn Platform>> {
    Ok(Box::new(crate::windows::WindowsService))
}

/// 当前操作系统的实现。
#[cfg(target_os = "linux")]
fn current_platform() -> Result<Box<dyn Platform>> {
    Ok(Box::new(crate::linux::LinuxService))
}

/// 当前操作系统的实现。
#[cfg(target_os = "macos")]
fn current_platform() -> Result<Box<dyn Platform>> {
    Ok(Box::new(crate::macos::MacosService))
}

/// 当前操作系统的实现。
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn current_platform() -> Result<Box<dyn Platform>> {
    Err(anyhow::anyhow!("当前操作系统不支持注册系统服务"))
}
