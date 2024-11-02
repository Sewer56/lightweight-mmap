use super::*;
use crate::util::get_allocation_granularity;
use core::ptr;
use libc::*;

pub(crate) fn create_mmap(
    fd: c_int,
    offset: u64,
    len: usize,
    protection: c_int,
) -> Result<(*mut c_void, usize, usize), MmapError> {
    // Special case for zero length
    if len == 0 {
        return Ok((ptr::null_mut(), 0, 0));
    }

    let page_size = get_allocation_granularity();
    let aligned_offset = offset & !(page_size as u64 - 1);
    let offset_adjustment = offset - aligned_offset;

    // Adjust length to account for page alignment
    let adjusted_len = len + (offset_adjustment as usize);

    let ptr = unsafe {
        mmap64(
            ptr::null_mut(),
            adjusted_len,
            protection,
            MAP_SHARED,
            fd,
            aligned_offset as i64,
        )
    };

    if ptr == MAP_FAILED {
        return Err(MmapError::failed_to_map_memory_unix(errno::errno().0));
    }

    Ok((ptr, offset_adjustment as usize, adjusted_len))
}
