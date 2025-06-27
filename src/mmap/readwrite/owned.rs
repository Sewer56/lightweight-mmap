use super::MmapError;
use crate::handles::readwrite::ReadWriteFileHandle;
use crate::mmap::readwrite::ReadWriteMmap;
use alloc::sync::Arc;
use core::mem::transmute;
use core::ops::{Deref, DerefMut};

/// An owned version of ReadWriteMmap that owns its file handle via Arc, making it safe to send across threads
/// and allowing multiple mappings to share the same handle.
pub struct OwnedReadWriteMmap {
    handle: Arc<ReadWriteFileHandle>,
    mmap: ReadWriteMmap<'static>, // Use 'static since we own the handle
}

// SAFETY: OwnedReadWriteMmap is Sync because file access does not have thread restrictions.
// SAFETY: OwnedReadWriteMmap is Send because it owns the handle via Arc.
unsafe impl Send for OwnedReadWriteMmap {}
unsafe impl Sync for OwnedReadWriteMmap {}

impl OwnedReadWriteMmap {
    /// Returns a reference to the underlying file handle
    pub fn handle(&self) -> Arc<ReadWriteFileHandle> {
        self.handle.clone()
    }

    /// Creates a new owned memory mapping from a file handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The file handle to map, wrapped in an Arc for shared ownership
    /// * `offset` - The offset into the file to start the mapping
    /// * `length` - The length of the mapping
    ///
    /// # Safety
    ///
    /// This function is unsafe because it creates a new memory mapping. The caller must ensure:
    /// - The handle is valid and has appropriate permissions
    /// - The offset and length are valid for the file
    /// - The mapping will not be used after the OwnedReadWriteMmap is dropped
    /// - Concurrent writes to the same region of the file may result in undefined behavior
    pub unsafe fn new(
        handle: Arc<ReadWriteFileHandle>,
        offset: u64,
        length: usize,
    ) -> Result<Self, MmapError> {
        // Create the mapping using a reference to the handle inside the Arc
        let mmap = ReadWriteMmap::new(&handle, offset, length)?;

        // Convert the mmap to use a 'static lifetime since we own the handle via Arc
        let mmap = transmute::<ReadWriteMmap<'_>, ReadWriteMmap<'static>>(mmap);

        Ok(Self { handle, mmap })
    }

    /// Creates a new owned memory mapping from a file handle, automatically wrapping it in an Arc.
    ///
    /// This is a convenience constructor that takes ownership of a ReadWriteFileHandle and wraps it in an Arc.
    ///
    /// # Arguments
    ///
    /// * `handle` - The file handle to map
    /// * `offset` - The offset into the file to start the mapping
    /// * `length` - The length of the mapping
    ///
    /// # Safety
    ///
    /// Same safety requirements as `new()`.
    pub unsafe fn from_handle(
        handle: ReadWriteFileHandle,
        offset: u64,
        length: usize,
    ) -> Result<Self, MmapError> {
        Self::new(Arc::new(handle), offset, length)
    }
}

impl Deref for OwnedReadWriteMmap {
    type Target = ReadWriteMmap<'static>;

    fn deref(&self) -> &Self::Target {
        &self.mmap
    }
}

impl DerefMut for OwnedReadWriteMmap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mmap
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn readonly_owned_mmap_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<OwnedReadWriteMmap>();
    }

    #[test]
    fn readonly_owned_mmap_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<OwnedReadWriteMmap>();
    }

    #[test]
    fn test_owned_mmap_basic_rw() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let handle = ReadWriteFileHandle::open(temp_file.path().to_str().unwrap()).unwrap();
        let mut mmap = unsafe { OwnedReadWriteMmap::from_handle(handle, 0, 13).unwrap() };

        assert_eq!(&mmap.as_slice()[0..5], b"Hello");

        // Test write capability
        mmap.as_mut_slice()[0..5].copy_from_slice(b"HELLO");
        assert_eq!(&mmap.as_slice()[0..5], b"HELLO");
    }

    #[test]
    fn test_owned_mmap_multiple_views_rw() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"First Second").unwrap();
        temp_file.flush().unwrap();

        let handle =
            Arc::new(ReadWriteFileHandle::open(temp_file.path().to_str().unwrap()).unwrap());

        let mut first_view = unsafe { OwnedReadWriteMmap::new(handle.clone(), 0, 5).unwrap() };
        let mut second_view = unsafe { OwnedReadWriteMmap::new(handle, 6, 6).unwrap() };

        first_view.as_mut_slice()[0..5].copy_from_slice(b"FIRST");
        second_view.as_mut_slice()[0..6].copy_from_slice(b"SECOND");

        assert_eq!(&first_view.as_slice(), b"FIRST");
        assert_eq!(&second_view.as_slice(), b"SECOND");
    }
}
