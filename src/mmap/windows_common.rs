use super::MmapError;
use super::*;
use crate::util::get_allocation_granularity;
use core::ptr::{self, null};
use core::{ffi::c_void, ptr::null_mut};
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::{Foundation::*, System::Memory::*};

pub(crate) fn create_view(
    handle: HANDLE,
    mapping: &mut HANDLE,
    offset: u64,
    len: usize,
    protection: u32,
    access: u32,
) -> Result<(*mut c_void, usize, usize), MmapError> {
    // Special case for zero length
    if len == 0 {
        return Ok((ptr::null_mut(), 0, 0));
    }

    // Lazily create the mapping if it doesn't exist
    if *mapping == INVALID_HANDLE_VALUE {
        *mapping = unsafe {
            let map = CreateFileMappingW(handle, null_mut(), protection, 0, 0, null());

            if map.is_null() {
                return Err(MmapError::failed_to_map_memory(GetLastError()));
            }
            map
        };
    }

    let page_size = get_allocation_granularity();
    let aligned_offset = offset & !(page_size as u64 - 1);
    let offset_adjustment = offset - aligned_offset;

    // Adjust length to account for page alignment
    let adjusted_len = len + (offset_adjustment as usize);

    let ptr = unsafe {
        MapViewOfFile(
            *mapping,
            access,
            (aligned_offset >> 32) as u32,
            aligned_offset as u32,
            adjusted_len,
        )
        .Value
    };

    if ptr.is_null() {
        return Err(MmapError::failed_to_map_memory(unsafe { GetLastError() }));
    }

    Ok((ptr, offset_adjustment as usize, adjusted_len))
}

pub(crate) fn advise_memory(addr: *mut core::ffi::c_void, len: usize, advice: MemoryAdvice) {
    // Windows only supports prefetching (similar to MADV_WILLNEED)
    // Other advice types are ignored
    if advice.contains(MemoryAdvice::WILL_NEED) {
        let entry = WIN32_MEMORY_RANGE_ENTRY {
            VirtualAddress: addr,
            NumberOfBytes: len,
        };

        unsafe {
            let _ = PrefetchVirtualMemory(GetCurrentProcess(), 1, &entry, 0);
        }
    }
}
