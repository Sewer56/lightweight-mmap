use crate::*;
use ::core::marker::PhantomData;
use ::core::ptr::null_mut;
use handles::HandleOpenError;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::*;
use Win32::Globalization::*;

/// Windows platform-specific implementation for ReadOnlyFileHandle.
pub struct InnerHandle {
    handle: HANDLE,
    _marker: PhantomData<()>,
}

impl InnerHandle {
    /// Opens the file with read-only access and shared permissions.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to open.
    ///
    /// # Errors
    ///
    /// Returns a `FileProviderError` if the file cannot be opened or the path conversion fails.
    pub fn open(path: &str) -> Result<Self, HandleOpenError> {
        let wide_path =
            to_wide(path).map_err(|code| HandleOpenError::failed_to_convert_path(code, path))?;

        let handle = unsafe {
            CreateFileW(
                wide_path.as_ptr(),
                GENERIC_READ,
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

/// Converts a `&str` to a wide string (`Box<[u16]>`) for Windows API using MultiByteToWideChar.
///
/// # Arguments
///
/// * `s` - The UTF-8 string slice to convert.
///
/// # Errors
///
/// Returns the Windows error code if the conversion fails.
fn to_wide(s: &str) -> Result<Box<[u16]>, u32> {
    let c_str = s.as_bytes();

    unsafe {
        // Determine the required buffer size
        let len = MultiByteToWideChar(
            65001, // CP_UTF8
            0,
            c_str.as_ptr(),
            s.len() as i32,
            null_mut(),
            0,
        );

        if len == 0 {
            return Err(GetLastError());
        }

        let mut buffer: Box<[u16]> = Box::new_uninit_slice(len as usize + 1).assume_init();

        // Perform the actual conversion
        let result = MultiByteToWideChar(
            65001, // CP_UTF8
            0,
            c_str.as_ptr(),
            s.len() as i32,
            buffer.as_mut_ptr(),
            len,
        );

        if result == 0 {
            return Err(GetLastError());
        }

        Ok(buffer)
    }
}
