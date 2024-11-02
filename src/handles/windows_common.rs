use super::HandleOpenError;
use crate::util::to_wide;
use core::{ffi::c_void, ptr::*};
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};

/// Opens the file with specified access and shared permissions.
///
/// # Arguments
///
/// * `path` - The path to the file to open.
/// * `access` - Desired access rights ([`GENERIC_READ`] or [`GENERIC_READ`] | [`GENERIC_WRITE`])
///
/// # Errors
///
/// Returns a `HandleOpenError` if the file cannot be opened or the path conversion fails.
pub(crate) fn open_with_access(path: &str, access: u32) -> Result<*mut c_void, HandleOpenError> {
    let wide_path =
        to_wide(path).map_err(|code| HandleOpenError::failed_to_convert_path(code, path))?;

    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            access,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        let error_code = unsafe { GetLastError() };
        return Err(HandleOpenError::failed_to_open_file_handle(
            error_code, path,
        ));
    }

    Ok(handle)
}

#[cfg(target_os = "windows")]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn get_file_size(handle: HANDLE) -> Result<i64, HandleOpenError> {
    let mut size = 0i64;
    let size_ptr = (&mut size) as *mut i64;
    let ret = unsafe { GetFileSizeEx(handle, size_ptr) };

    if ret == 0 {
        Err(HandleOpenError::FailedToGetFileSize(unsafe {
            GetLastError()
        }))
    } else {
        Ok(size)
    }
}
