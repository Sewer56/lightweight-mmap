use super::*;
use core::marker::PhantomData;
use libc::*;
use unix_common::create_mmap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ReadOnlyMmapInner<'a> {
    ptr: *mut c_void,
    length: usize,
    _phantom: PhantomData<&'a ReadOnlyFileHandle>,
}

unsafe impl Send for ReadOnlyMmapInner<'_> {}

impl<'a> ReadOnlyMmapInner<'a> {
    pub(crate) fn new(
        handle: &'a ReadOnlyFileHandle,
        offset: u64,
        len: usize,
    ) -> Result<(Self, usize, usize), MmapError> {
        let (ptr, offset_adjustment, adjusted_len) =
            create_mmap(handle.handle().fd(), offset, len, PROT_READ)?;

        Ok((
            ReadOnlyMmapInner {
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

impl Drop for ReadOnlyMmapInner<'_> {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                munmap(self.ptr, self.length);
            }
        }
    }
}
