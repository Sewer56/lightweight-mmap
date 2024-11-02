use core::ptr::null_mut;
use windows_sys::Win32::{Foundation::*, Globalization::*};

/// Converts a `&str` to a wide string (`Box<[u16]>`) for Windows API using MultiByteToWideChar.
///
/// # Arguments
///
/// * `s` - The UTF-8 string slice to convert.
///
/// # Errors
///
/// Returns the Windows error code if the conversion fails.
pub fn to_wide(s: &str) -> Result<Box<[u16]>, u32> {
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

        // Write null terminator
        *buffer.get_unchecked_mut(len as usize) = 0;
        Ok(buffer)
    }
}
