use super::HandleOpenError;
use alloc::ffi::CString;
use core::mem::zeroed;
use libc::*;

/// Default file permissions: rw-r--r-- (644)
const DEFAULT_FILE_MODE: mode_t = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;

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

    // If O_CREAT is in flags, provide mode, otherwise mode is ignored
    let fd = unsafe {
        if flags & O_CREAT != 0 {
            open(c_path.as_ptr(), flags, DEFAULT_FILE_MODE as c_uint)
        } else {
            open(c_path.as_ptr(), flags)
        }
    };

    if fd < 0 {
        return Err(HandleOpenError::failed_to_open_file_handle_unix(
            errno::errno().0,
            path,
        ));
    }

    Ok(fd)
}

#[cfg(unix)]
#[allow(clippy::unnecessary_cast)] // st_size type varies across Unix platforms
pub fn get_file_size(fd: c_int) -> Result<i64, HandleOpenError> {
    #[cfg(target_env = "gnu")]
    {
        let mut st: libc::stat64 = unsafe { zeroed() };
        let ret = unsafe { fstat64(fd, &mut st) };

        if ret == -1 {
            Err(HandleOpenError::FailedToGetFileSize(errno::errno().0))
        } else {
            Ok(st.st_size as i64)
        }
    }

    #[cfg(not(target_env = "gnu"))]
    {
        let mut st: libc::stat = unsafe { zeroed() };
        let ret = unsafe { fstat(fd, &mut st) };

        if ret == -1 {
            Err(HandleOpenError::FailedToGetFileSize(errno::errno().0))
        } else {
            Ok(st.st_size as i64)
        }
    }
}

#[cfg(unix)]
#[allow(clippy::comparison_chain)]
pub(crate) fn set_file_size(fd: c_int, size: i64) -> Result<(), HandleOpenError> {
    unsafe {
        // Get current file size
        let current_size = get_file_size(fd)?;

        if size > current_size {
            #[cfg(target_os = "linux")]
            {
                // Linux specific fallocate - allocates without zeroing
                if fallocate(fd, 0, 0, size as off_t) != 0 {
                    return Err(HandleOpenError::failed_to_set_file_size(errno::errno().0));
                }
            }
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            {
                // Some BSDs support posix_fallocate
                let ret = libc::posix_fallocate(fd, 0, size as off_t);
                if ret != 0 {
                    return Err(HandleOpenError::failed_to_set_file_size(ret));
                }
            }
            #[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly")))]
            {
                // Other Unix systems including macOS - have to use ftruncate
                if ftruncate(fd, size as off_t) != 0 {
                    return Err(HandleOpenError::failed_to_set_file_size(errno::errno().0));
                }
            }
        } else if size < current_size {
            // Shrink using ftruncate on all platforms
            if ftruncate(fd, size as off_t) != 0 {
                return Err(HandleOpenError::failed_to_set_file_size(errno::errno().0));
            }
        }
    }
    Ok(())
}
