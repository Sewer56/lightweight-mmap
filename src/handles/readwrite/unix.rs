use super::*;
use core::marker::PhantomData;
use libc::*;
use unix_common::*;

/// Unix platform-specific implementation for [`ReadWriteFileHandle`].
pub struct InnerHandle {
    fd: c_int,
    _marker: PhantomData<()>,
}

impl InnerHandle {
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

    pub fn create_preallocated(path: &str, size: i64) -> Result<Self, HandleOpenError> {
        let fd = open_with_flags(path, O_RDWR | O_CREAT)?;
        if let Err(e) = set_file_size(fd, size) {
            unsafe { close(fd) };
            return Err(e);
        }

        Ok(InnerHandle {
            fd,
            _marker: PhantomData,
        })
    }
}

impl Drop for InnerHandle {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}
