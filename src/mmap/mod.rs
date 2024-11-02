pub mod error;
pub mod readonly;
pub mod readwrite;

use bitflags::bitflags;
pub use error::*;
pub use readonly::*;
pub use readwrite::*;

#[cfg(unix)]
pub mod unix_common;
#[cfg(target_os = "windows")]
pub mod windows_common;

#[cfg(unix)]
use unix_common::*;
#[cfg(target_os = "windows")]
use windows_common::*;

#[cfg(feature = "trim-file-lengths")]
use crate::handles::*;

#[cfg(feature = "trim-file-lengths")]
pub(crate) fn adjust_len_to_file_size(
    file_size: Result<i64, HandleOpenError>,
    offset: u64,
    len: usize,
) -> Result<usize, MmapError> {
    let file_size = file_size.map_err(|_| MmapError::failed_to_get_file_size())?;

    if offset >= file_size as u64 {
        return Ok(0);
    }

    let remaining = file_size as u64 - offset;
    Ok(remaining.min(len as u64) as usize)
}

bitflags! {
    /// Memory advice options that can be given to the operating system.
    /// These are hints and may be combined using bitwise operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MemoryAdvice: u32 {
        /// Indicates that the application expects to access the memory soon
        const WILL_NEED = 0b001;
        /// Indicates that memory access will be sequential from lower to higher addresses
        const SEQUENTIAL = 0b010;
        /// Indicates that memory access will be random (non-sequential)
        const RANDOM = 0b100;
    }
}
