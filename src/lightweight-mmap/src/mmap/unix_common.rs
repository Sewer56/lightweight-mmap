use super::*;
use crate::util::get_allocation_granularity;
use core::ptr::{self, NonNull};
use libc::*;

pub(crate) fn create_mmap(
    fd: c_int,
    offset: u64,
    len: usize,
    protection: c_int,
) -> Result<(*mut c_void, usize, usize), MmapError> {
    // Special case for zero length
    if len == 0 {
        return Ok((NonNull::dangling().as_ptr(), 0, 0));
    }

    let page_size = get_allocation_granularity();
    let aligned_offset = offset & !(page_size as u64 - 1);
    let offset_adjustment = offset - aligned_offset;
    let aligned_offset = aligned_offset as i64; // Re-cast as signed

    // Adjust length to account for page alignment
    let adjusted_len = len + (offset_adjustment as usize);

    let ptr = unsafe {
        #[cfg(target_env = "gnu")]
        {
            mmap64(
                ptr::null_mut(),
                adjusted_len,
                protection,
                MAP_SHARED,
                fd,
                aligned_offset as libc::off64_t,
            )
        }
        #[cfg(not(target_env = "gnu"))]
        {
            mmap(
                ptr::null_mut(),
                adjusted_len,
                protection,
                MAP_SHARED,
                fd,
                aligned_offset as libc::off_t,
            )
        }
    };

    if ptr == MAP_FAILED {
        return Err(MmapError::failed_to_map_memory_unix(errno::errno().0));
    }

    Ok((ptr, offset_adjustment as usize, adjusted_len))
}

#[cfg(unix)]
pub(crate) fn advise_memory(addr: *mut libc::c_void, len: usize, advice: MemoryAdvice) {
    // Check each flag and make the corresponding madvise call
    // Ignore any errors as these are just hints
    unsafe {
        if advice.contains(MemoryAdvice::WILL_NEED) {
            let _ = madvise(addr, len, MADV_WILLNEED);
        }
        if advice.contains(MemoryAdvice::SEQUENTIAL) {
            let _ = madvise(addr, len, MADV_SEQUENTIAL);
        }
        if advice.contains(MemoryAdvice::RANDOM) {
            let _ = madvise(addr, len, MADV_RANDOM);
        }
    }
}
