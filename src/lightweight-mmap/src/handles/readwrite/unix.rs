use super::*;
use libc::*;
use unix_common::*;

/// Unix platform-specific implementation for [`ReadWriteFileHandle`].
pub struct InnerHandle {
    fd: c_int,
}

unsafe impl Sync for InnerHandle {}
unsafe impl Send for InnerHandle {}

impl InnerHandle {
    #[cfg(feature = "std")]
    pub fn open(path: &std::path::Path) -> Result<Self, HandleOpenError> {
        let path_str = path.to_str().ok_or_else(|| {
            HandleOpenError::failed_to_open_file_handle_unix(-1, "<invalid_utf8>")
        })?;
        let fd = open_with_flags(path_str, O_RDWR)?;
        Ok(InnerHandle { fd })
    }

    #[cfg(not(feature = "std"))]
    pub fn open(path: &str) -> Result<Self, HandleOpenError> {
        let fd = open_with_flags(path, O_RDWR)?;
        Ok(InnerHandle { fd })
    }

    /// Returns the raw file descriptor.
    pub fn fd(&self) -> c_int {
        self.fd
    }

    #[cfg(feature = "std")]
    pub fn create_preallocated(path: &std::path::Path, size: i64) -> Result<Self, HandleOpenError> {
        let path_str = path.to_str().ok_or_else(|| {
            HandleOpenError::failed_to_open_file_handle_unix(-1, "<invalid_utf8>")
        })?;
        let fd = open_with_flags(path_str, O_RDWR | O_CREAT)?;
        if let Err(e) = set_file_size(fd, size) {
            unsafe { close(fd) };
            return Err(e);
        }

        Ok(InnerHandle { fd })
    }

    #[cfg(not(feature = "std"))]
    pub fn create_preallocated(path: &str, size: i64) -> Result<Self, HandleOpenError> {
        let fd = open_with_flags(path, O_RDWR | O_CREAT)?;
        if let Err(e) = set_file_size(fd, size) {
            unsafe { close(fd) };
            return Err(e);
        }

        Ok(InnerHandle { fd })
    }
}

impl Drop for InnerHandle {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}
