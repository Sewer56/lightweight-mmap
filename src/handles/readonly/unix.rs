use super::*;
use libc::*;
use unix_common::open_with_flags;

/// Unix platform-specific implementation for [`ReadOnlyFileHandle`].
pub struct InnerHandle {
    fd: c_int,
}

unsafe impl Sync for InnerHandle {}
unsafe impl Send for InnerHandle {}

impl InnerHandle {
    /// Opens the file with read-only access.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to open.
    ///
    /// # Errors
    ///
    /// Returns a [`HandleOpenError`] if the file cannot be opened.
    #[cfg(feature = "std")]
    pub fn open(path: &std::path::Path) -> Result<Self, HandleOpenError> {
        let path_str = path.to_str().ok_or_else(|| {
            HandleOpenError::failed_to_open_file_handle_unix(-1, "<invalid_utf8>")
        })?;
        let fd = open_with_flags(path_str, O_RDONLY)?;
        Ok(InnerHandle { fd })
    }

    /// Opens the file with read-only access.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to open.
    ///
    /// # Errors
    ///
    /// Returns a [`HandleOpenError`] if the file cannot be opened.
    #[cfg(not(feature = "std"))]
    pub fn open(path: &str) -> Result<Self, HandleOpenError> {
        let fd = open_with_flags(path, O_RDONLY)?;
        Ok(InnerHandle { fd })
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
