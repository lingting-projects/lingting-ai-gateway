//! 运行期目录：可由启动参数覆盖，未指定时按应用目录生成。

use std::path::PathBuf;

use crate::APP_ID;
use anyhow::Result;
use framework_core::ApplicationDirectory;

/// pglite 数据目录名，位于应用目录的 data 下。
const PGLITE_DIRECTORY: &str = "pgsql";

/// 运行期目录。
#[derive(Debug, Clone)]
pub struct Directorys {
    /// pglite 数据目录。
    pub pglite: PathBuf,
    /// 日志目录。
    pub logs: PathBuf,
}

impl Directorys {
    /// 解析运行期目录：指定了就用指定的，未指定则按应用目录生成。
    ///
    /// 服务方式启动时运行身份与手动运行不同，应用目录会落到系统账号下，
    /// 因此注册服务时把当前目录作为启动参数写入，保证两种启动方式共用同一份数据与日志。
    pub fn resolve(pglite: Option<PathBuf>, logs: Option<PathBuf>) -> Result<Self> {
        let directory = ApplicationDirectory::root(APP_ID)?;

        Ok(Self {
            pglite: pglite.unwrap_or_else(|| directory.data.join(PGLITE_DIRECTORY)),
            logs: logs.unwrap_or(directory.logs),
        })
    }
}
