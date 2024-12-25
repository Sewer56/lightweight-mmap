use super::MmapError;
use crate::handles::readonly::ReadOnlyFileHandle;
use crate::mmap::readonly::ReadOnlyMmap;
use alloc::sync::Arc;
use core::{
    mem::transmute,
    ops::{Deref, DerefMut},
};

/// An owned version of ReadOnlyMmap that owns its file handle via Arc, making it safe to send across threads
/// and allowing multiple mappings to share the same handle.
pub struct OwnedReadOnlyMmap {
    handle: Arc<ReadOnlyFileHandle>,
    mmap: ReadOnlyMmap<'static>, // Use 'static since we own the handle
}

// SAFETY: OwnedReadOnlyMmap is Sync because file access does not have thread restrictions.
// SAFETY: OwnedReadOnlyMmap is Send because it owns the handle via Arc.
unsafe impl Send for OwnedReadOnlyMmap {}
unsafe impl Sync for OwnedReadOnlyMmap {}

impl OwnedReadOnlyMmap {
    /// Returns a reference to the underlying file handle
    pub fn handle(&self) -> Arc<ReadOnlyFileHandle> {
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
    /// - The mapping will not be used after the OwnedReadOnlyMmap is dropped
    pub unsafe fn new(
        handle: Arc<ReadOnlyFileHandle>,
        offset: u64,
        length: usize,
    ) -> Result<Self, MmapError> {
        // Create the mapping using a reference to the handle inside the Arc
        let mmap = ReadOnlyMmap::new(&handle, offset, length)?;

        // Convert the mmap to use a 'static lifetime since we own the handle via Arc
        let mmap = transmute::<ReadOnlyMmap<'_>, ReadOnlyMmap<'static>>(mmap);

        Ok(Self { handle, mmap })
    }

    /// Creates a new owned memory mapping from a file handle, automatically wrapping it in an Arc.
    ///
    /// This is a convenience constructor that takes ownership of a ReadOnlyFileHandle and wraps it in an Arc.
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
        handle: ReadOnlyFileHandle,
        offset: u64,
        length: usize,
    ) -> Result<Self, MmapError> {
        Self::new(Arc::new(handle), offset, length)
    }
}

impl Deref for OwnedReadOnlyMmap {
    type Target = ReadOnlyMmap<'static>;

    fn deref(&self) -> &Self::Target {
        &self.mmap
    }
}

impl DerefMut for OwnedReadOnlyMmap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mmap
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn readonly_owned_mmap_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<OwnedReadOnlyMmap>();
    }

    #[test]
    fn readonly_owned_mmap_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<OwnedReadOnlyMmap>();
    }

    #[test]
    fn test_owned_mmap_basic() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let temp_file_path = temp_file.path().to_str().unwrap();
        let handle = ReadOnlyFileHandle::open(temp_file_path).unwrap();
        let mmap = unsafe { OwnedReadOnlyMmap::from_handle(handle, 0, 13).unwrap() };

        assert_eq!(&mmap.as_slice()[0..5], b"Hello");
        assert_eq!(&mmap.as_slice()[7..12], b"World");
    }

    #[test]
    fn test_owned_mmap_multiple_views() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"First Second").unwrap();
        temp_file.flush().unwrap();

        let handle =
            Arc::new(ReadOnlyFileHandle::open(temp_file.path().to_str().unwrap()).unwrap());

        let first_view = unsafe { OwnedReadOnlyMmap::new(handle.clone(), 0, 5).unwrap() };
        let second_view = unsafe { OwnedReadOnlyMmap::new(handle, 6, 6).unwrap() };

        assert_eq!(&first_view.as_slice(), b"First");
        assert_eq!(&second_view.as_slice(), b"Second");
    }
}
