#![allow(clippy::missing_safety_doc)]

use crate::handles::ReadOnlyFileHandle;
use std::ffi::{CStr, CString};
use std::os::raw::*;

// These exports are only used to measure library size after build.
// This is not 'proper' code.

#[repr(C)]
pub struct HandleResult {
    success: bool,
    data: HandleResultData,
}

#[repr(C)]
pub union HandleResultData {
    handle: *mut ReadOnlyFileHandle,
    error: *const c_char,
}

#[no_mangle]
pub unsafe extern "C" fn open_read_handle(path: *const c_char) -> HandleResult {
    // Safety check for null pointer
    if path.is_null() {
        return HandleResult {
            success: false,
            data: HandleResultData {
                error: c"Null path pointer provided".as_ptr(),
            },
        };
    }

    // Convert C string to Rust string
    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            return HandleResult {
                success: false,
                data: HandleResultData {
                    error: c"Invalid UTF-8 in path".as_ptr(),
                },
            }
        }
    };

    // Try to open the handle
    match ReadOnlyFileHandle::open(path_str) {
        Ok(handle) => HandleResult {
            success: true,
            data: HandleResultData {
                handle: Box::into_raw(Box::new(handle)),
            },
        },
        Err(e) => HandleResult {
            success: false,
            data: HandleResultData {
                error: e.to_string().as_ptr() as *const i8, // yes, this is a bug. We're only doing size testing here.
            },
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_handle(handle: *mut ReadOnlyFileHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_error(error: *mut c_char) {
    if !error.is_null() {
        unsafe {
            drop(CString::from_raw(error));
        }
    }
}