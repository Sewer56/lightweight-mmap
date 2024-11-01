use super::*;
use core::marker::PhantomData;
use libc::*;
use std::ffi::CString;

/// Unix platform-specific implementation for ReadOnlyFileHandle.
pub struct InnerHandle {
    fd: c_int,
    _marker: PhantomData<()>,
}

impl InnerHandle {
    /// Opens the file with read-only access.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to open.
    ///
    /// # Errors
    ///
    /// Returns a `HandleOpenError` if the file cannot be opened.
    pub fn open(path: &str) -> Result<Self, HandleOpenError> {
        let c_path = CString::new(path)
            .map_err(|_| HandleOpenError::failed_to_open_file_handle_unix(-1, path))?;

        let fd = unsafe { open(c_path.as_ptr(), O_RDONLY) };
        if fd < 0 {
            return Err(HandleOpenError::failed_to_open_file_handle_unix(
                errno(),
                path,
            ));
        }

        Ok(InnerHandle {
            fd,
            _marker: PhantomData,
        })
    }

    /// Returns the raw file descriptor.
    pub fn fd(&self) -> c_int {
        self.fd
    }
}

impl Drop for InnerHandle {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}

/// Retrieves the current value of errno.
///
/// # Returns
///
/// The current errno value.
fn errno() -> i32 {
    unsafe { *libc::__errno_location() }
}
