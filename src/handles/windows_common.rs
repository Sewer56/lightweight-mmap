use super::HandleOpenError;
#[cfg(not(feature = "std"))]
use crate::util::to_wide;
use core::{ffi::c_void, ptr::*};
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};

/// Opens the file with specified access and shared permissions.
///
/// # Arguments
///
/// * `path` - The path to the file to open.
/// * `access` - Desired access rights ([`GENERIC_READ`] or [`GENERIC_READ`] | [`GENERIC_WRITE`])
/// * `creation` - Creation disposition (e.g., [`CREATE_NEW`], [`OPEN_EXISTING`])
///
/// # Errors
///
/// Returns a `HandleOpenError` if the file cannot be opened or the path conversion fails.
#[cfg(feature = "std")]
pub(crate) fn open_with_access(
    path: &std::path::Path,
    access: u32,
    creation: u32,
) -> Result<*mut c_void, HandleOpenError> {
    // Convert Path to wide string directly via OsStr
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::vec::Vec;
    let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();

    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            access,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            null_mut(),
            creation,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        let error_code = unsafe { GetLastError() };
        let path_str = path.to_string_lossy();
        return Err(HandleOpenError::failed_to_open_file_handle(
            error_code, &path_str,
        ));
    }

    Ok(handle)
}

/// Opens the file with specified access and shared permissions.
///
/// # Arguments
///
/// * `path` - The path to the file to open.
/// * `access` - Desired access rights ([`GENERIC_READ`] or [`GENERIC_READ`] | [`GENERIC_WRITE`])
/// * `creation` - Creation disposition (e.g., [`CREATE_NEW`], [`OPEN_EXISTING`])
///
/// # Errors
///
/// Returns a `HandleOpenError` if the file cannot be opened or the path conversion fails.
#[cfg(not(feature = "std"))]
pub(crate) fn open_with_access(
    path: &str,
    access: u32,
    creation: u32,
) -> Result<*mut c_void, HandleOpenError> {
    let wide_path =
        to_wide(path).map_err(|code| HandleOpenError::failed_to_convert_path(code, path))?;

    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            access,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            null_mut(),
            creation,
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

#[cfg(target_os = "windows")]
pub(crate) fn set_file_size(handle: HANDLE, size: i64) -> Result<(), HandleOpenError> {
    let mut distance_high = ((size >> 32) & 0xFFFFFFFF) as i32;
    let distance_low = (size & 0xFFFFFFFF) as i32;

    unsafe {
        let result = SetFilePointer(handle, distance_low, &mut distance_high, FILE_BEGIN);
        if result == INVALID_SET_FILE_POINTER && GetLastError() != 0 {
            return Err(HandleOpenError::failed_to_set_file_size(GetLastError()));
        }

        if SetEndOfFile(handle) == 0 {
            return Err(HandleOpenError::failed_to_set_file_size(GetLastError()));
        }

        // Reset file pointer to beginning
        SetFilePointer(handle, 0, null_mut(), FILE_BEGIN);
    }
    Ok(())
}
