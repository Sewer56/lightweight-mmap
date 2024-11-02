use core::fmt::*;

/// Represents errors that can occur when providing file data.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "no-format"), derive(thiserror::Error))]
pub enum HandleOpenError {
    /// Failed to convert the file path to a wide string (Windows only).
    #[cfg(target_os = "windows")]
    #[cfg_attr(
        all(not(feature = "no-format"), debug_assertions),
        error("Failed to convert path to wide string. Error code: {0}, Path: {1}")
    )]
    #[cfg_attr(
        all(not(feature = "no-format"), not(debug_assertions)),
        error("Failed to convert path to wide string. Error code: {0}")
    )]
    FailedToConvertPath(u32, #[cfg(debug_assertions)] String),

    /// Failed to open the file handle.
    #[cfg(target_os = "windows")]
    #[cfg_attr(
        all(not(feature = "no-format"), debug_assertions),
        error("Failed to open file handle. Error code: {0}, Path: {1}")
    )]
    #[cfg_attr(
        all(not(feature = "no-format"), not(debug_assertions)),
        error("Failed to open file handle. Error code: {0}")
    )]
    FailedToOpenFileHandle(u32, #[cfg(debug_assertions)] String),

    /// Failed to open the file handle (Unix).
    #[cfg(unix)]
    #[cfg_attr(
        all(not(feature = "no-format"), debug_assertions),
        error("Failed to open file handle. Error code: {0}, Path: {1}")
    )]
    #[cfg_attr(
        all(not(feature = "no-format"), not(debug_assertions)),
        error("Failed to open file handle. Error code: {0}")
    )]
    FailedToOpenFileHandle(i32, #[cfg(debug_assertions)] String),

    /// Failed to get file size.
    #[cfg(target_os = "windows")]
    #[cfg_attr(
        not(feature = "no-format"),
        error("Failed to get file size. Error code: {0}")
    )]
    FailedToGetFileSize(u32),

    /// Failed to get file size (Unix).
    #[cfg(unix)]
    #[cfg_attr(
        not(feature = "no-format"),
        error("Failed to get file size. Error code: {0}")
    )]
    FailedToGetFileSize(i32),
}

impl HandleOpenError {
    #[cfg(target_os = "windows")]
    #[allow(unused_variables)]
    pub fn failed_to_convert_path(err_code: u32, path: &str) -> Self {
        #[cfg(debug_assertions)]
        {
            Self::FailedToConvertPath(err_code, path.to_string())
        }

        #[cfg(not(debug_assertions))]
        {
            Self::FailedToConvertPath(err_code)
        }
    }

    #[cfg(target_os = "windows")]
    #[allow(unused_variables)]
    pub fn failed_to_open_file_handle(err_code: u32, path: &str) -> Self {
        #[cfg(debug_assertions)]
        {
            Self::FailedToOpenFileHandle(err_code, path.to_string())
        }

        #[cfg(not(debug_assertions))]
        {
            Self::FailedToOpenFileHandle(err_code)
        }
    }

    #[cfg(unix)]
    #[allow(unused_variables)]
    pub fn failed_to_open_file_handle_unix(err_code: i32, path: &str) -> Self {
        #[cfg(debug_assertions)]
        {
            Self::FailedToOpenFileHandle(err_code, path.to_string())
        }

        #[cfg(not(debug_assertions))]
        {
            Self::FailedToOpenFileHandle(err_code)
        }
    }
}

#[cfg(feature = "no-format")]
impl Display for HandleOpenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use itoa::*;
        use nanokit::string_concat_unsafe::*;

        match self {
            #[cfg(all(target_os = "windows", debug_assertions))]
            Self::FailedToConvertPath(code, path) => {
                let mut buffer = Buffer::new();
                let code_str = buffer.format(*code);
                let error_msg = unsafe {
                    concat_3_no_overflow(
                        "Failed to convert path to wide string. Error code: ",
                        code_str,
                        concat_2_no_overflow(", Path: ", path),
                    )
                };
                f.write_str(&error_msg)
            }

            #[cfg(all(target_os = "windows", not(debug_assertions)))]
            Self::FailedToConvertPath(code) => {
                let mut buffer = Buffer::new();
                let code_str = buffer.format(*code);
                let error_msg = unsafe {
                    concat_2_no_overflow(
                        "Failed to convert path to wide string. Error code: ",
                        code_str,
                    )
                };
                f.write_str(&error_msg)
            }

            #[cfg(all(target_os = "windows", debug_assertions))]
            Self::FailedToOpenFileHandle(code, path) => {
                let mut buffer = Buffer::new();
                let code_str = buffer.format(*code);
                let error_msg = unsafe {
                    concat_3_no_overflow(
                        "Failed to open file handle. Error code: ",
                        code_str,
                        concat_2_no_overflow(", Path: ", path),
                    )
                };
                f.write_str(&error_msg)
            }

            #[cfg(all(target_os = "windows", not(debug_assertions)))]
            Self::FailedToOpenFileHandle(code) => {
                let mut buffer = Buffer::new();
                let code_str = buffer.format(*code);
                let error_msg = unsafe {
                    concat_2_no_overflow("Failed to open file handle. Error code: ", code_str)
                };
                f.write_str(&error_msg)
            }

            #[cfg(all(unix, debug_assertions))]
            Self::FailedToOpenFileHandle(code, path) => {
                let mut buffer = Buffer::new();
                let code_str = buffer.format(*code);
                let error_msg = unsafe {
                    concat_3_no_overflow(
                        "Failed to open file handle. Error code: ",
                        code_str,
                        concat_2_no_overflow(", Path: ", path),
                    )
                };
                f.write_str(&error_msg)
            }

            #[cfg(all(unix, not(debug_assertions)))]
            Self::FailedToOpenFileHandle(code) => {
                let mut buffer = Buffer::new();
                let code_str = buffer.format(*code);
                let error_msg = unsafe {
                    concat_2_no_overflow("Failed to open file handle. Error code: ", code_str)
                };
                f.write_str(&error_msg)
            }

            #[cfg(target_os = "windows")]
            Self::FailedToGetFileSize(code) => {
                let mut buffer = Buffer::new();
                let code_str = buffer.format(*code);
                let error_msg = unsafe {
                    concat_2_no_overflow("Failed to get file size. Error code: ", code_str)
                };
                f.write_str(&error_msg)
            }

            #[cfg(unix)]
            Self::FailedToGetFileSize(code) => {
                let mut buffer = Buffer::new();
                let code_str = buffer.format(*code);
                let error_msg = unsafe {
                    concat_2_no_overflow("Failed to get file size. Error code: ", code_str)
                };
                f.write_str(&error_msg)
            }
        }
    }
}
