use super::MmapError;
use crate::util::get_allocation_granularity;
use core::ptr::{self, null};
use core::{ffi::c_void, ptr::null_mut};
use windows_sys::Win32::{Foundation::*, System::Memory::*};

pub(crate) fn create_mmap(
    handle: HANDLE,
    offset: u64,
    len: usize,
    protection: u32,
    access: u32,
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

    unsafe {
        let mapping = CreateFileMappingW(handle, null_mut(), protection, 0, 0, null());

        if mapping.is_null() {
            return Err(MmapError::failed_to_map_memory(GetLastError()));
        }

        let ptr = MapViewOfFile(
            mapping,
            access,
            (aligned_offset >> 32) as u32,
            aligned_offset as u32,
            adjusted_len,
        )
        .Value;

        CloseHandle(mapping);

        if ptr.is_null() {
            return Err(MmapError::failed_to_map_memory(GetLastError()));
        }

        Ok((ptr, offset_adjustment as usize, adjusted_len))
    }
}
