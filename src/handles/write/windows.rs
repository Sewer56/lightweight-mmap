use crate::*;
use ::core::marker::PhantomData;
use ::core::ptr::null_mut;
use handles::HandleOpenError;
use util::to_wide;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Storage::FileSystem::*;

/// Windows platform-specific implementation for ReadWriteFileHandle.
pub struct InnerHandle {
    handle: HANDLE,
    _marker: PhantomData<()>,
}

impl InnerHandle {
    /// Opens the file with read-write access and shared permissions.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to open.
    ///
    /// # Errors
    ///
    /// Returns a `HandleOpenError` if the file cannot be opened or the path conversion fails.
    pub fn open(path: &str) -> Result<Self, HandleOpenError> {
        let wide_path =
            to_wide(path).map_err(|code| HandleOpenError::failed_to_convert_path(code, path))?;

        let handle = unsafe {
            CreateFileW(
                wide_path.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            let error_code = unsafe { GetLastError() };
            return Err(HandleOpenError::failed_to_open_file_handle(
                error_code, path,
            ));
        }

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
