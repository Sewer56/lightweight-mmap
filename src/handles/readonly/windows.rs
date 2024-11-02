// This should replace both readwrite/windows.rs and readonly/windows.rs:

use super::windows_common::*;
use crate::*;
use ::core::marker::PhantomData;
use handles::HandleOpenError;
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};

/// Windows platform-specific implementation for [`ReadOnlyFileHandle`].
pub struct InnerHandle {
    handle: HANDLE,
    _marker: PhantomData<()>,
}

impl InnerHandle {
    /// Opens the file with appropriate access.
    pub fn open(path: &str) -> Result<Self, HandleOpenError> {
        let handle = open_with_access(path, GENERIC_READ, OPEN_EXISTING)?;
        Ok(InnerHandle {
            handle,
            _marker: PhantomData,
        })
    }

    /// Returns the raw HANDLE.
    pub fn handle(&self) -> HANDLE {
        self.handle
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
