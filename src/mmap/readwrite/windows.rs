use super::*;
use core::{ffi::c_void, marker::PhantomData};
use windows_common::create_mmap;
use windows_sys::Win32::System::Memory::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ReadWriteMmapInner<'a> {
    ptr: *mut c_void,
    _phantom: PhantomData<&'a ReadWriteFileHandle>,
}

impl<'a> ReadWriteMmapInner<'a> {
    pub(crate) fn new(
        handle: &'a ReadWriteFileHandle,
        offset: u64,
        len: usize,
    ) -> Result<(Self, usize, usize), MmapError> {
        let (ptr, offset_adjustment, adjusted_len) = create_mmap(
            handle.handle().handle(),
            offset,
            len,
            PAGE_READWRITE,
            FILE_MAP_WRITE,
        )?;

        Ok((
            ReadWriteMmapInner {
                ptr,
                _phantom: PhantomData,
            },
            offset_adjustment,
            adjusted_len,
        ))
    }

    #[inline]
    pub fn data(&self) -> *mut c_void {
        self.ptr
    }
}

impl Drop for ReadWriteMmapInner<'_> {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS { Value: self.ptr });
            }
        }
    }
}
