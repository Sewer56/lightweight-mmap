use super::*;
use crate::handles::ReadOnlyFileHandle;
use core::slice::from_raw_parts;

#[cfg(unix)]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(unix)]
use unix::*;
#[cfg(target_os = "windows")]
use windows::*;

/// A read-only memory mapping that allows shared access to a file's contents.
///
/// This struct provides a safe wrapper around platform-specific memory mapping
/// implementations, allowing read-only access to a file's contents through memory
/// mapping. The mapping can be created with a specific offset and length.
///
/// The mapping cannot outlive the file handle it was created from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReadOnlyMmap<'a> {
    inner: ReadOnlyMmapInner<'a>,
    offset_adjustment: usize,
    length: usize,
}

unsafe impl Send for ReadOnlyMmap<'_> {}

impl<'a> ReadOnlyMmap<'a> {
    /// Creates a new read-only memory mapping for the specified file handle.
    ///
    /// # Arguments
    ///
    /// * `handle` - The file handle to create the mapping from
    /// * `offset` - The offset into the file where the mapping should begin
    /// * `len` - The length of the mapping in bytes
    ///
    /// # Errors
    ///
    /// Returns a `MmapError` if:
    /// - The mapping cannot be created
    /// - The length is zero
    /// - The offset and length would exceed the file size
    /// - The system cannot allocate the required resources
    pub fn new(handle: &'a ReadOnlyFileHandle, offset: u64, len: usize) -> Result<Self, MmapError> {
        #[cfg(feature = "trim-file-lengths")]
        let len = adjust_len_to_file_size(handle.size(), offset, len)?;

        let (inner, offset_adjustment, length) = ReadOnlyMmapInner::new(handle, offset, len)?;
        Ok(ReadOnlyMmap {
            inner,
            offset_adjustment,
            length,
        })
    }

    /// Returns a slice of the mapped memory.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it creates a slice from raw pointers.
    /// The caller must ensure that:
    /// - The memory is not modified by other parts of the program while this slice exists
    /// - The lifetime of the slice does not exceed the lifetime of the mapping
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { from_raw_parts(self.data(), self.len()) }
    }

    /// Returns a raw pointer to the mapped memory.
    ///
    /// The returned pointer is adjusted for the requested offset, accounting for
    /// any page alignment requirements.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory is accessed within the bounds of
    /// the mapping and that no other threads are modifying the file while it
    /// is mapped.
    #[inline]
    pub fn data(&self) -> *const u8 {
        unsafe { (self.inner.data() as *const u8).add(self.offset_adjustment) }
    }

    /// Returns the length of the mapped region in bytes.
    ///
    /// This returns the originally requested length, not including any padding
    /// added for page alignment.
    #[inline]
    pub fn len(&self) -> usize {
        self.length - self.offset_adjustment
    }

    /// Returns whether the mapping is empty (zero length).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Provides advice to the operating system about how the memory mapping will be accessed.
    ///
    /// # Arguments
    ///
    /// * `advice` - Bit flags indicating the expected access patterns for this memory region
    ///
    /// # Note
    ///
    /// This is a hint to the operating system and may be ignored. Not all advice types
    /// are supported on all platforms. On Windows, only [`MemoryAdvice::WILL_NEED`] has an effect.
    /// Multiple advice flags can be combined using bitwise operations.
    pub fn advise(&self, advice: MemoryAdvice) {
        if !self.is_empty() {
            advise_memory(self.inner.data(), self.length, advice)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn readonly_mmap_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ReadOnlyMmap<'_>>();
    }

    #[test]
    fn can_create_empty_mapping() {
        let file = NamedTempFile::new().unwrap();
        let handle = ReadOnlyFileHandle::open(file.path().to_str().unwrap()).unwrap();

        let mapping = ReadOnlyMmap::new(&handle, 0, 0).unwrap();
        assert_eq!(mapping.len(), 0);
        assert!(mapping.is_empty());
    }

    #[test]
    fn empty_mapping_returns_null() {
        let file = NamedTempFile::new().unwrap();
        let handle = ReadOnlyFileHandle::open(file.path().to_str().unwrap()).unwrap();

        let mapping = ReadOnlyMmap::new(&handle, 0, 0).unwrap();
        assert!(mapping.data().is_null());
    }

    #[test]
    fn can_map_entire_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello, World!").unwrap();
        file.flush().unwrap();

        let handle = ReadOnlyFileHandle::open(file.path().to_str().unwrap()).unwrap();
        let mapping = ReadOnlyMmap::new(&handle, 0, 13).unwrap();

        assert_eq!(mapping.len(), 13);
        let data = mapping.as_slice();
        assert_eq!(data, b"Hello, World!");
    }

    #[test]
    fn can_map_with_offset() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello, World!").unwrap();
        file.flush().unwrap();

        let handle = ReadOnlyFileHandle::open(file.path().to_str().unwrap()).unwrap();
        let mapping = ReadOnlyMmap::new(&handle, 7, 5).unwrap();

        assert_eq!(mapping.len(), 5);
        let data = mapping.as_slice();
        assert_eq!(data, b"World");
    }

    #[test]
    fn can_map_with_unaligned_offset() {
        let mut file = NamedTempFile::new().unwrap();
        // Write enough data to cross page boundaries
        let data = vec![0u8; 8192];
        file.write_all(&data).unwrap();
        file.flush().unwrap();

        let handle = ReadOnlyFileHandle::open(file.path().to_str().unwrap()).unwrap();
        // Try mapping with an unaligned offset
        let mapping = ReadOnlyMmap::new(&handle, 4099, 1000).unwrap();

        assert_eq!(mapping.len(), 1000);
        let mapped_data = mapping.as_slice();
        assert_eq!(mapped_data, &data[4099..4099 + 1000]);
    }

    // Tests that only run with trim-file-lengths feature
    #[cfg(feature = "trim-file-lengths")]
    mod trim_lengths {
        use super::*;

        #[test]
        fn mapping_is_trimmed_to_file_size() {
            let mut file = NamedTempFile::new().unwrap();
            file.write_all(b"Hello").unwrap();
            file.flush().unwrap();

            let handle = ReadOnlyFileHandle::open(file.path().to_str().unwrap()).unwrap();
            let mapping = ReadOnlyMmap::new(&handle, 0, 10).unwrap();

            assert_eq!(mapping.len(), 5); // Should be trimmed to actual file size
            let data = unsafe { from_raw_parts(mapping.data(), mapping.len()) };
            assert_eq!(data, b"Hello");
        }

        #[test]
        fn offset_beyond_file_size_creates_empty_mapping() {
            let mut file = NamedTempFile::new().unwrap();
            file.write_all(b"Hello").unwrap();
            file.flush().unwrap();

            let handle = ReadOnlyFileHandle::open(file.path().to_str().unwrap()).unwrap();
            let mapping = ReadOnlyMmap::new(&handle, 10, 1).unwrap();

            assert_eq!(mapping.len(), 0);
            assert!(mapping.is_empty());
        }

        #[test]
        fn partial_mapping_at_end_of_file() {
            let mut file = NamedTempFile::new().unwrap();
            file.write_all(b"Hello, World!").unwrap();
            file.flush().unwrap();

            let handle = ReadOnlyFileHandle::open(file.path().to_str().unwrap()).unwrap();
            let mapping = ReadOnlyMmap::new(&handle, 11, 10).unwrap(); // Request more than available

            assert_eq!(mapping.len(), 2); // Only "d!" should be mapped
            let data = unsafe { from_raw_parts(mapping.data(), mapping.len()) };
            assert_eq!(data, b"d!");
        }
    }
}
