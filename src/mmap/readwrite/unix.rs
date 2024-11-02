use super::*;
use core::{marker::PhantomData, ptr};
use libc::*;
use unix_common::create_mmap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ReadWriteMmapInner<'a> {
    ptr: *mut c_void,
    length: usize,
    _phantom: PhantomData<&'a ReadWriteFileHandle>,
}

impl<'a> ReadWriteMmapInner<'a> {
    pub(crate) fn new(
        handle: &'a ReadWriteFileHandle,
        offset: u64,
        len: usize,
    ) -> Result<(Self, usize, usize), MmapError> {
        let (ptr, offset_adjustment, adjusted_len) =
            create_mmap(handle.handle().fd(), offset, len, PROT_READ | PROT_WRITE)?;

        Ok((
            ReadWriteMmapInner {
                ptr,
                length: adjusted_len,
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

impl<'a> Drop for ReadWriteMmapInner<'a> {
    fn drop(&mut self) {
        unsafe {
            if self.ptr != ptr::null_mut() {
                munmap(self.ptr, self.length);
            }
        }
    }
}
