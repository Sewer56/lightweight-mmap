use super::*;
use core::marker::PhantomData;
use libc::*;
use unix_common::open_with_flags;

/// Unix platform-specific implementation for [`ReadWriteFileHandle`].
pub struct InnerHandle {
    fd: c_int,
    _marker: PhantomData<()>,
}

impl InnerHandle {
    /// Opens the file with read-write access.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to open.
    ///
    /// # Errors
    ///
    /// Returns a [`HandleOpenError`] if the file cannot be opened.
    pub fn open(path: &str) -> Result<Self, HandleOpenError> {
        let fd = open_with_flags(path, O_RDWR)?;
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
