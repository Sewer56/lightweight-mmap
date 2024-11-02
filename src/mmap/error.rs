/// Represents errors that can occur during memory mapping.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "no-format"), derive(thiserror::Error))]
pub enum MmapError {
    /// Failed to map memory on Windows.
    #[cfg(target_os = "windows")]
    #[cfg_attr(
        all(not(feature = "no-format"), debug_assertions),
        error("Failed to map memory on Windows. Error code: {0}")
    )]
    #[cfg_attr(
        all(not(feature = "no-format"), not(debug_assertions)),
        error("Failed to map memory on Windows. Error code: {0}")
    )]
    FailedToMapMemory(u32),

    /// Failed to map memory on Unix.
    #[cfg(unix)]
    #[cfg_attr(
        all(not(feature = "no-format"), debug_assertions),
        error("Failed to map memory on Unix. Error code: {0}")
    )]
    #[cfg_attr(
        all(not(feature = "no-format"), not(debug_assertions)),
        error("Failed to map memory on Unix. Error code: {0}")
    )]
    FailedToMapMemory(i32),

    /// Generic mapping failure with a message.
    #[cfg_attr(
        all(not(feature = "no-format"), debug_assertions),
        error("Mapping failed: {0}")
    )]
    #[cfg_attr(
        all(not(feature = "no-format"), not(debug_assertions)),
        error("Mapping failed.")
    )]
    MappingFailed(String),

    /// Failed to get file size.
    #[cfg_attr(
        all(not(feature = "no-format"), debug_assertions),
        error("Failed to get file size")
    )]
    #[cfg_attr(
        all(not(feature = "no-format"), not(debug_assertions)),
        error("Failed to get file size")
    )]
    FailedToGetFileSize,
}

impl MmapError {
    #[cfg(target_os = "windows")]
    pub fn failed_to_map_memory(error_code: u32) -> Self {
        MmapError::FailedToMapMemory(error_code)
    }

    #[cfg(unix)]
    pub fn failed_to_map_memory_unix(error_code: i32) -> Self {
        MmapError::FailedToMapMemory(error_code)
    }

    pub fn mapping_failed(message: &str) -> Self {
        MmapError::MappingFailed(message.to_string())
    }

    pub fn failed_to_get_file_size() -> Self {
        MmapError::FailedToGetFileSize
    }
}

#[cfg(feature = "no-format")]
impl std::fmt::Display for MmapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use itoa::*;
        use nanokit::string_concat_unsafe::*;

        match self {
            MmapError::FailedToMapMemory(code) => {
                let mut buffer = Buffer::new();
                let code_str = buffer.format(*code);
                let error_msg =
                    unsafe { concat_2_no_overflow("Failed to map memory. Error code: ", code_str) };
                f.write_str(&error_msg)
            }

            MmapError::MappingFailed(msg) => {
                let error_msg = unsafe { concat_2_no_overflow("Mapping failed: ", msg) };
                f.write_str(&error_msg)
            }

            MmapError::FailedToGetFileSize => f.write_str("Failed to get file size"),
        }
    }
}
