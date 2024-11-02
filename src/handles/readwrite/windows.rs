// This should replace both readwrite/windows.rs and readonly/windows.rs:

use super::windows_common::*;
use crate::*;
use ::core::marker::PhantomData;
use handles::HandleOpenError;
use windows_sys::Win32::{Foundation::*, Storage::FileSystem::*};

#[cfg(feature = "mmap")]
use core::cell::UnsafeCell;

/// Windows platform-specific implementation for [`ReadWriteFileHandle`].
pub struct InnerHandle {
    handle: HANDLE,
    #[cfg(feature = "mmap")]
    pub(crate) mapping: UnsafeCell<HANDLE>,
    _marker: PhantomData<()>,
}

unsafe impl Send for InnerHandle {}

impl InnerHandle {
    /// Opens the file with appropriate access.
    pub fn open(path: &str) -> Result<Self, HandleOpenError> {
        let handle = open_with_access(path, GENERIC_READ | GENERIC_WRITE, OPEN_EXISTING)?;

        Ok(InnerHandle {
            handle,
            #[cfg(feature = "mmap")]
            mapping: UnsafeCell::new(INVALID_HANDLE_VALUE),
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
            #[cfg(feature = "mmap")]
            mapping: UnsafeCell::new(INVALID_HANDLE_VALUE),
            _marker: PhantomData,
        })
    }
}

impl Drop for InnerHandle {
    fn drop(&mut self) {
        unsafe {
            #[cfg(feature = "mmap")]
            {
                let mapping = *self.mapping.get_mut();
                if mapping != INVALID_HANDLE_VALUE {
                    CloseHandle(mapping);
                }
            }

            if self.handle != INVALID_HANDLE_VALUE {
                CloseHandle(self.handle);
            }
        }
    }
}
