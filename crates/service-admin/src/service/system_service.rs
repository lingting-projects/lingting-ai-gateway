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
    ///
    /// `home` 表示该账户实际的 OS Home/Profile Directory。
    /// ```
    pub fn users() -> Result<Vec<SystemUserVO>> {
        let users = platform::list()?;
        let filter = users
            .into_iter()
            .filter(|(_, home)| {
                let path = Path::new(home);
                let exists = path.exists();
                exists && path.is_dir()
            })
            .map(|(name, home)| SystemUserVO { name, home })
            .collect();
        Ok(filter)
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

/// Windows 平台实现。
///
/// Windows 不能通过：
///
/// ```text
/// %SystemDrive%\Users\<username>
/// ```
///
/// 推导用户实际 Profile Directory。
///
/// ```text
/// 用户名
///   ↓
/// LookupAccountNameW
///   ↓
/// SID
///   ↓
/// HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\<SID>
///   ↓
/// ProfileImagePath
/// ```
///
/// 获取实际 Profile Directory。
#[cfg(windows)]
mod platform {
    use std::ffi::c_void;
    use std::ptr;

    use anyhow::{Result, anyhow};

    use windows_sys::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_NONE_MAPPED};
    use windows_sys::Win32::Security::{LookupAccountNameW, SID_NAME_USE};
    use windows_sys::Win32::System::Environment::ExpandEnvironmentStringsW;
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_EXPAND_SZ, REG_SZ, RegCloseKey, RegOpenKeyExW,
        RegQueryValueExW,
    };

    use windows_sys::Win32::NetworkManagement::NetManagement::{
        FILTER_NORMAL_ACCOUNT, MAX_PREFERRED_LENGTH, NetApiBufferFree, NetUserEnum, USER_INFO_0,
    };

    use super::BUILT_IN_USERS;

    const PROFILE_LIST_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList";

    const PROFILE_IMAGE_PATH: &str = "ProfileImagePath";

    /// 枚举本地普通账户，并获取每个账户实际 Profile Directory。
    pub(super) fn list() -> Result<Vec<(String, String)>> {
        local_accounts()
    }

    /// 枚举本机普通账户。
    fn local_accounts() -> Result<Vec<(String, String)>> {
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

        let mut users = Vec::new();

        if !buffer.is_null() && entries_read > 0 {
            let entries = unsafe {
                std::slice::from_raw_parts(buffer.cast::<USER_INFO_0>(), entries_read as usize)
            };

            for entry in entries {
                let name = unsafe { wide_string(entry.usri0_name) };

                if name.is_empty() || is_built_in(&name) {
                    continue;
                }

                // 获取实际 Profile Directory。
                //
                // 如果账户没有对应的 ProfileList 项，则跳过。
                if let Some(home) = profile_directory(&name)? {
                    users.push((name, home));
                }
            }
        }

        unsafe {
            NetApiBufferFree(buffer.cast());
        }

        Ok(users)
    }

    /// 获取 Windows 用户实际 Profile Directory。
    ///
    /// 例如：
    ///
    /// ```text
    /// C:\Users\alice
    /// D:\Profiles\alice
    /// ```
    /// ```
    fn profile_directory(username: &str) -> Result<Option<String>> {
        let sid = match account_sid(username)? {
            Some(sid) => sid,
            None => return Ok(None),
        };

        let sid_string = sid_to_string(&sid)?;

        let key_path = format!(r"{PROFILE_LIST_KEY}\{sid_string}");

        let key_path = wide_string_with_null(&key_path);

        let mut key: HKEY = ptr::null_mut();

        let status =
            unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, key_path.as_ptr(), 0, KEY_READ, &mut key) };

        if status != 0 {
            return Ok(None);
        }

        let result = query_profile_image_path(key);

        unsafe {
            RegCloseKey(key);
        }

        result
    }

    /// 根据 Windows 账户名获取 SID。
    fn account_sid(username: &str) -> Result<Option<Vec<u8>>> {
        let account_name = wide_string_with_null(username);

        // 第一次调用只获取所需 buffer 大小。
        let mut sid_size = 0u32;
        let mut domain_size = 0u32;
        let mut sid_type = SID_NAME_USE::default();

        let ok = unsafe {
            LookupAccountNameW(
                ptr::null(),
                account_name.as_ptr(),
                ptr::null_mut(),
                &mut sid_size,
                ptr::null_mut(),
                &mut domain_size,
                &mut sid_type,
            )
        };

        if ok != 0 {
            return Err(anyhow!("LookupAccountNameW 第一次调用意外成功"));
        }

        let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };

        if error == ERROR_NONE_MAPPED {
            return Ok(None);
        }

        if error != ERROR_INSUFFICIENT_BUFFER {
            return Err(anyhow!(
                "获取用户 `{username}` SID 大小失败，错误码：{error}"
            ));
        }

        let mut sid = vec![0u8; sid_size as usize];
        let mut domain = vec![0u16; domain_size as usize];

        let ok = unsafe {
            LookupAccountNameW(
                ptr::null(),
                account_name.as_ptr(),
                sid.as_mut_ptr().cast::<c_void>(),
                &mut sid_size,
                domain.as_mut_ptr(),
                &mut domain_size,
                &mut sid_type,
            )
        };

        if ok == 0 {
            let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };

            return Err(anyhow!("获取用户 `{username}` SID 失败，错误码：{error}"));
        }

        sid.truncate(sid_size as usize);

        Ok(Some(sid))
    }

    /// 将 SID 转成：
    ///
    /// ```text
    /// S-1-5-21-...
    /// ```
    fn sid_to_string(sid: &[u8]) -> Result<String> {
        if sid.len() < 8 {
            return Err(anyhow!("SID 数据长度无效"));
        }
        let revision = sid[0];
        let sub_authority_count = sid[1] as usize;
        let expected_size = 8usize
            .checked_add(
                sub_authority_count
                    .checked_mul(4)
                    .ok_or_else(|| anyhow!("SID 子权限数量溢出"))?,
            )
            .ok_or_else(|| anyhow!("SID 长度溢出"))?;
        if sid.len() < expected_size {
            return Err(anyhow!("SID 数据不完整"));
        }
        // SID_IDENTIFIER_AUTHORITY 是 6 字节，大端序。
        let authority = ((sid[2] as u64) << 40)
            | ((sid[3] as u64) << 32)
            | ((sid[4] as u64) << 24)
            | ((sid[5] as u64) << 16)
            | ((sid[6] as u64) << 8)
            | sid[7] as u64;
        let mut result = format!("S-{revision}-{authority}");
        for index in 0..sub_authority_count {
            let offset = 8 + index * 4;
            let value = u32::from_le_bytes([
                sid[offset],
                sid[offset + 1],
                sid[offset + 2],
                sid[offset + 3],
            ]);
            result.push('-');
            result.push_str(&value.to_string());
        }
        Ok(result)
    }
    /// 从 ProfileList 注册表项读取 ProfileImagePath。
    ///
    /// 支持：
    ///
    /// - REG_SZ
    /// - REG_EXPAND_SZ
    ///
    /// 例如：
    ///
    /// ```text
    /// %SystemDrive%\Users\alice
    /// ```
    ///
    /// 会被展开成：
    ///
    /// ```text
    /// C:\Users\alice
    /// ```
    fn query_profile_image_path(key: HKEY) -> Result<Option<String>> {
        let value_name = wide_string_with_null(PROFILE_IMAGE_PATH);

        let mut value_type = 0u32;
        let mut value_size = 0u32;

        let status = unsafe {
            RegQueryValueExW(
                key,
                value_name.as_ptr(),
                ptr::null_mut(),
                &mut value_type,
                ptr::null_mut(),
                &mut value_size,
            )
        };

        if status != 0 {
            return Ok(None);
        }

        if value_size == 0 {
            return Ok(None);
        }

        let mut buffer = vec![0u8; value_size as usize];

        let status = unsafe {
            RegQueryValueExW(
                key,
                value_name.as_ptr(),
                ptr::null_mut(),
                &mut value_type,
                buffer.as_mut_ptr(),
                &mut value_size,
            )
        };

        if status != 0 {
            return Err(anyhow!("读取 ProfileImagePath 失败，错误码：{status}"));
        }

        let mut path = utf16_buffer_to_string(&buffer);

        if path.is_empty() {
            return Ok(None);
        }

        // ProfileImagePath 通常为 REG_EXPAND_SZ。
        if value_type == REG_EXPAND_SZ {
            path = expand_environment_strings(&path)?;
        } else if value_type != REG_SZ {
            return Ok(None);
        }

        Ok(Some(path))
    }

    /// 展开 Windows 环境变量。
    fn expand_environment_strings(value: &str) -> Result<String> {
        let input = wide_string_with_null(value);

        let required = unsafe { ExpandEnvironmentStringsW(input.as_ptr(), ptr::null_mut(), 0) };

        if required == 0 {
            let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };

            return Err(anyhow!(
                "展开 ProfileImagePath 环境变量失败，错误码：{error}"
            ));
        }

        let mut output = vec![0u16; required as usize];

        let length =
            unsafe { ExpandEnvironmentStringsW(input.as_ptr(), output.as_mut_ptr(), required) };

        if length == 0 {
            let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };

            return Err(anyhow!(
                "展开 ProfileImagePath 环境变量失败，错误码：{error}"
            ));
        }

        // API 返回值包含结尾 NULL。
        output.truncate(length.saturating_sub(1) as usize);

        Ok(String::from_utf16_lossy(&output))
    }

    /// 是否为 Windows 内置账户。
    fn is_built_in(name: &str) -> bool {
        BUILT_IN_USERS
            .iter()
            .any(|built_in| built_in.eq_ignore_ascii_case(name))
    }

    /// 读取以 0 结尾的 UTF-16 字符串。
    ///
    /// 空指针返回空串。
    unsafe fn wide_string(value: *const u16) -> String {
        if value.is_null() {
            return String::new();
        }

        let mut length = 0usize;

        while unsafe { *value.add(length) } != 0 {
            length += 1;
        }

        String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(value, length) })
    }

    /// 将 Rust 字符串转换成以 0 结尾的 UTF-16 字符串。
    fn wide_string_with_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// 将 Windows Registry 中的 UTF-16 buffer 转成 Rust String。
    fn utf16_buffer_to_string(buffer: &[u8]) -> String {
        let utf16 = buffer
            .as_chunks::<2>().0.iter()
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take_while(|&value| value != 0)
            .collect::<Vec<_>>();

        String::from_utf16_lossy(&utf16)
    }
}

/// Unix 平台实现。
///
/// Unix 下 `pw_dir` 就是账户数据库记录的实际 Home Directory。
#[cfg(unix)]
mod platform {
    use std::ffi::CStr;

    use anyhow::Result;

    /// 通过 getpwent 枚举账户，并读取系统记录中的 Home Directory。
    pub(super) fn list() -> Result<Vec<(String, String)>> {
        let mut users = Vec::new();

        unsafe {
            libc::setpwent();

            loop {
                let entry = libc::getpwent();

                if entry.is_null() {
                    break;
                }

                let name = c_string((*entry).pw_name);
                let home = c_string((*entry).pw_dir);

                if !name.is_empty() && !home.is_empty() {
                    users.push((name, home));
                }
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
