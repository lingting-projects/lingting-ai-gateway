use anyhow::Result;

#[cfg(unix)]
pub fn is_elevated() -> Result<bool> {
    Ok(unsafe { libc::geteuid() } == 0)
}

#[cfg(windows)]
pub fn is_elevated() -> Result<bool> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0u32;
        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );
        let last_error = if result == 0 {
            Some(std::io::Error::last_os_error())
        } else {
            None
        };
        CloseHandle(token);
        if let Some(error) = last_error {
            return Err(error.into());
        }
        Ok(elevation.TokenIsElevated != 0)
    }
}
#[cfg(not(any(unix, windows)))]
pub fn is_elevated() -> Result<bool> {
    Ok(false)
}
