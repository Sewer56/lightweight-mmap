// This should replace both readwrite/windows.rs and readonly/windows.rs:

use super::windows_common::*;
use crate::*;
use ::core::marker::PhantomData;
use handles::HandleOpenError;
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};

/// Windows platform-specific implementation for [`ReadWriteFileHandle`].
pub struct InnerHandle {
    handle: HANDLE,
    _marker: PhantomData<()>,
}

impl InnerHandle {
    /// Opens the file with appropriate access.
    pub fn open(path: &str) -> Result<Self, HandleOpenError> {
        let handle = open_with_access(path, GENERIC_READ | GENERIC_WRITE, OPEN_EXISTING)?;
        Ok(InnerHandle {
            handle,
            _marker: PhantomData,
        })
    }

    /// Returns the raw HANDLE.
    pub fn handle(&self) -> HANDLE {
        self.handle
    }

    pub fn create_preallocated(path: &str, size: i64) -> Result<Self, HandleOpenError> {
        let handle = open_with_access(path, GENERIC_READ | GENERIC_WRITE, OPEN_ALWAYS)?;

        if let Err(e) = set_file_size(handle, size) {
            unsafe { CloseHandle(handle) };
            return Err(e);
        }

        Ok(InnerHandle {
            handle,
            _marker: PhantomData,
        })
    }
}

impl Drop for InnerHandle {
    fn drop(&mut self) {
        unsafe {
            if self.handle != INVALID_HANDLE_VALUE {
                CloseHandle(self.handle);
            }
        }
    }
}
