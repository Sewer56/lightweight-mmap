use super::HandleOpenError;
use core::mem::zeroed;
use libc::*;
use std::ffi::CString;

/// Opens the file with specified access mode.
///
/// # Arguments
///
/// * `path` - The path to the file to open.
/// * `flags` - Open flags (O_RDONLY or O_RDWR)
///
/// # Errors
///
/// Returns a `HandleOpenError` if the file cannot be opened.
pub(crate) fn open_with_flags(path: &str, flags: c_int) -> Result<c_int, HandleOpenError> {
    let c_path = CString::new(path)
        .map_err(|_| HandleOpenError::failed_to_open_file_handle_unix(-1, path))?;

    let fd = unsafe { open(c_path.as_ptr(), flags) };

    if fd < 0 {
        return Err(HandleOpenError::failed_to_open_file_handle_unix(
            errno::errno().0,
            path,
        ));
    }

    Ok(fd)
}

#[cfg(unix)]
pub fn get_file_size(fd: c_int) -> Result<i64, HandleOpenError> {
    let mut st: libc::stat = unsafe { zeroed() };
    let ret = unsafe { fstat(fd, &mut st) };

    if ret == -1 {
        Err(HandleOpenError::FailedToGetFileSize(errno::errno().0))
    } else {
        Ok(st.st_size)
    }
}
