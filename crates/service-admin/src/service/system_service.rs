//! 系统信息服务：账户列表等本机信息。
//!
//! 只读取本机状态、不访问数据库，因此没有对应的表与 Repository。

use std::path::Path;

use anyhow::Result;
use types_admin::dto::SystemUserVO;

/// 系统信息服务。
pub struct SystemService;

impl SystemService {
    /// 列出在本机存在家目录的账户。
    pub fn users() -> Result<Vec<SystemUserVO>> {
        let users = platform::list()?;

        Ok(users
            .into_iter()
            .filter(|(_, home)| Path::new(home).is_dir())
            .map(|(name, home)| SystemUserVO { name, home })
            .collect())
    }
}

/// Windows 内置账户，不属于使用者，直接排除。
#[cfg(windows)]
const BUILT_IN_USERS: &[&str] = &[
    "Guest",
    "DefaultAccount",
    "WDAGUtilityAccount",
    "Default",
    "Default User",
    "All Users",
    "Public",
];

/// 平台实现：返回账户名与其家目录，家目录是否存在由调用方判定。
#[cfg(windows)]
mod platform {
    use std::path::PathBuf;
    use std::ptr;

    use anyhow::{Result, anyhow};
    use windows_sys::Win32::NetworkManagement::NetManagement::{
        FILTER_NORMAL_ACCOUNT, MAX_PREFERRED_LENGTH, NetApiBufferFree, NetUserEnum, USER_INFO_0,
    };

    use super::BUILT_IN_USERS;

    /// 枚举本地账户；家目录按 Windows 约定取 `%SystemDrive%\Users\<账户名>`。
    pub(super) fn list() -> Result<Vec<(String, String)>> {
        let users_dir = users_directory();

        Ok(local_account_names()?
            .into_iter()
            .filter(|name| !name.is_empty() && !is_built_in(name))
            .map(|name| {
                let home = users_dir.join(&name);

                (name, home.to_string_lossy().into_owned())
            })
            .collect())
    }

    /// 读取本机全部普通账户名。
    fn local_account_names() -> Result<Vec<String>> {
        let mut buffer = ptr::null_mut();
        let mut entries_read = 0u32;
        let mut total_entries = 0u32;
        let status = unsafe {
            NetUserEnum(
                ptr::null(),
                0,
                FILTER_NORMAL_ACCOUNT,
                &mut buffer,
                MAX_PREFERRED_LENGTH,
                &mut entries_read,
                &mut total_entries,
                ptr::null_mut(),
            )
        };
        if status != 0 {
            return Err(anyhow!("枚举本机账户失败，错误码：{status}"));
        }

        let mut names = Vec::new();
        if !buffer.is_null() && entries_read > 0 {
            let entries = unsafe {
                std::slice::from_raw_parts(buffer.cast::<USER_INFO_0>(), entries_read as usize)
            };
            names = entries
                .iter()
                .map(|entry| unsafe { wide_string(entry.usri0_name) })
                .collect();
        }
        unsafe { NetApiBufferFree(buffer.cast()) };

        Ok(names)
    }

    /// 本机用户目录：`%SystemDrive%\Users`，取不到盘符时按 `C:` 处理。
    fn users_directory() -> PathBuf {
        let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());

        PathBuf::from(drive).join("Users")
    }

    /// 是否为 Windows 内置账户。
    fn is_built_in(name: &str) -> bool {
        BUILT_IN_USERS
            .iter()
            .any(|built_in| built_in.eq_ignore_ascii_case(name))
    }

    /// 读取以 0 结尾的 UTF-16 字符串；空指针返回空串。
    unsafe fn wide_string(value: *const u16) -> String {
        if value.is_null() {
            return String::new();
        }

        let mut length = 0;
        while unsafe { *value.add(length) } != 0 {
            length += 1;
        }

        String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(value, length) })
    }
}

/// 平台实现：返回账户名与其家目录，家目录是否存在由调用方判定。
#[cfg(unix)]
mod platform {
    use std::ffi::CStr;

    use anyhow::Result;

    /// 通过 `getpwent` 枚举账户，取 `pw_name` 与 `pw_dir`。
    pub(super) fn list() -> Result<Vec<(String, String)>> {
        let mut users = Vec::new();
        unsafe {
            libc::setpwent();
            loop {
                let entry = libc::getpwent();
                if entry.is_null() {
                    break;
                }
                users.push((c_string((*entry).pw_name), c_string((*entry).pw_dir)));
            }
            libc::endpwent();
        }

        Ok(users)
    }

    /// 读取 C 字符串；空指针视为空串。
    unsafe fn c_string(value: *const libc::c_char) -> String {
        if value.is_null() {
            return String::new();
        }

        unsafe { CStr::from_ptr(value) }
            .to_string_lossy()
            .into_owned()
    }
}

/// 其他系统：返回空列表。
#[cfg(not(any(unix, windows)))]
mod platform {
    use anyhow::Result;

    pub(super) fn list() -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }
}
