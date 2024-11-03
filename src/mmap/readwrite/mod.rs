use super::*;
use crate::handles::ReadWriteFileHandle;
use core::slice::from_raw_parts;

#[cfg(unix)]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(unix)]
use unix::*;
#[cfg(target_os = "windows")]
use windows::*;

/// A read-write memory mapping that allows shared access to a file's contents.
///
/// This struct provides a safe wrapper around platform-specific memory mapping
/// implementations, allowing read and write access to a file's contents through
/// memory mapping. The mapping can be created with a specific offset and length.
///
/// The mapping cannot outlive the file handle it was created from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReadWriteMmap<'a> {
    inner: ReadWriteMmapInner<'a>,
    offset_adjustment: usize,
    length: usize,
}

unsafe impl Send for ReadWriteMmap<'_> {}

impl<'a> ReadWriteMmap<'a> {
    /// Creates a new read-write memory mapping for the specified file handle.
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
    pub fn new(
        handle: &'a ReadWriteFileHandle,
        offset: u64,
        len: usize,
    ) -> Result<Self, MmapError> {
        #[cfg(feature = "trim-file-lengths")]
        let len = adjust_len_to_file_size(handle.size(), offset, len)?;

        let (inner, offset_adjustment, length) = ReadWriteMmapInner::new(handle, offset, len)?;
        Ok(ReadWriteMmap {
            inner,
            offset_adjustment,
            length,
        })
    }

    /// Returns a slice of the mapped memory.
    /// The lifetime of the slice is the same as the mapping.
    #[inline]
    pub fn as_slice(&self) -> &'a [u8] {
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
    /// the mapping. The caller must also ensure the pointer does not outlast the
    /// lifetime of the mapping. It is recommended you use [`Self::as_slice`] instead
    /// for compiler enforced safety.
    #[inline]
    pub fn data(&self) -> *mut u8 {
        unsafe { (self.inner.data() as *mut u8).add(self.offset_adjustment) }
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
    use core::slice::from_raw_parts_mut;
    use std::{
        fs::File,
        io::{Read, Write},
    };
    use tempfile::NamedTempFile;

    #[test]
    fn readwrite_mmap_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ReadWriteMmap<'_>>();
    }

    #[test]
    fn can_create_empty_mapping() {
        let file = NamedTempFile::new().unwrap();
        let handle = ReadWriteFileHandle::open(file.path().to_str().unwrap()).unwrap();

        let mapping = ReadWriteMmap::new(&handle, 0, 0).unwrap();
        assert_eq!(mapping.len(), 0);
        assert!(mapping.is_empty());
    }

    #[test]
    fn empty_mapping_returns_null() {
        let file = NamedTempFile::new().unwrap();
        let handle = ReadWriteFileHandle::open(file.path().to_str().unwrap()).unwrap();

        let mapping = ReadWriteMmap::new(&handle, 0, 0).unwrap();
        assert!(mapping.data().is_null());
    }

    #[test]
    fn can_create_and_write_mapping() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello, World!").unwrap();
        file.flush().unwrap();

        let handle = ReadWriteFileHandle::open(file.path().to_str().unwrap()).unwrap();
        let mapping = ReadWriteMmap::new(&handle, 0, 13).unwrap();

        assert_eq!(mapping.len(), 13);
        unsafe {
            let data = from_raw_parts_mut(mapping.data(), mapping.len());
            data[0..5].copy_from_slice(b"HELLO");
        }

        // Read back the modified content
        let mut content = String::new();
        File::open(file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "HELLO, World!");
    }

    #[test]
    fn can_write_with_offset() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello, World!").unwrap();
        file.flush().unwrap();

        let handle = ReadWriteFileHandle::open(file.path().to_str().unwrap()).unwrap();
        let mapping = ReadWriteMmap::new(&handle, 7, 5).unwrap();

        assert_eq!(mapping.len(), 5);
        unsafe {
            let data = from_raw_parts_mut(mapping.data(), mapping.len());
            data.copy_from_slice(b"WORLD");
        }

        let mut content = String::new();
        File::open(file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "Hello, WORLD!");
    }

    #[test]
    fn can_write_with_unaligned_offset() {
        let mut file = NamedTempFile::new().unwrap();
        // Write enough data to cross page boundaries
        let mut data = vec![0u8; 8192];
        (0..data.len()).for_each(|i| {
            data[i] = (i % 256) as u8;
        });
        file.write_all(&data).unwrap();
        file.flush().unwrap();

        let handle = ReadWriteFileHandle::open(file.path().to_str().unwrap()).unwrap();
        // Try mapping with an unaligned offset
        let mapping = ReadWriteMmap::new(&handle, 4099, 1000).unwrap();

        assert_eq!(mapping.len(), 1000);
        unsafe {
            let mapped_data = from_raw_parts_mut(mapping.data(), mapping.len());
            // Modify the mapped data
            (0..mapped_data.len()).for_each(|i| {
                mapped_data[i] = 0xFF;
            });
        }

        // Verify the changes
        let mut new_data = Vec::new();
        File::open(file.path())
            .unwrap()
            .read_to_end(&mut new_data)
            .unwrap();

        // Check that only the mapped portion was modified
        assert_eq!(&new_data[..4099], &data[..4099]);
        assert_eq!(&new_data[4099..4099 + 1000], &vec![0xFF; 1000]);
        assert_eq!(&new_data[4099 + 1000..], &data[4099 + 1000..]);
    }

    #[test]
    fn handles_zero_length_edge_case() {
        let file = NamedTempFile::new().unwrap();
        let handle = ReadWriteFileHandle::open(file.path().to_str().unwrap()).unwrap();

        let result = ReadWriteMmap::new(&handle, 0, 0).unwrap();

        assert_eq!(result.len(), 0);
        assert!(result.is_empty());
    }

    #[test]
    fn multiple_mappings_same_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello, World!").unwrap();
        file.flush().unwrap();

        let handle = ReadWriteFileHandle::open(file.path().to_str().unwrap()).unwrap();
        let mapping1 = ReadWriteMmap::new(&handle, 0, 5).unwrap();
        let mapping2 = ReadWriteMmap::new(&handle, 7, 5).unwrap();

        unsafe {
            let data1 = from_raw_parts_mut(mapping1.data(), mapping1.len());
            let data2 = from_raw_parts_mut(mapping2.data(), mapping2.len());

            data1.copy_from_slice(b"HELLO");
            data2.copy_from_slice(b"WORLD");
        }

        let mut content = String::new();
        File::open(file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "HELLO, WORLD!");
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

            let handle = ReadWriteFileHandle::open(file.path().to_str().unwrap()).unwrap();
            let mapping = ReadWriteMmap::new(&handle, 0, 10).unwrap();

            assert_eq!(mapping.len(), 5);
            unsafe {
                let data = from_raw_parts_mut(mapping.data(), mapping.len());
                data.copy_from_slice(b"HELLO");
            }

            let mut content = String::new();
            File::open(file.path())
                .unwrap()
                .read_to_string(&mut content)
                .unwrap();
            assert_eq!(content, "HELLO");
        }

        #[test]
        fn offset_beyond_file_size_creates_empty_mapping() {
            let mut file = NamedTempFile::new().unwrap();
            file.write_all(b"Hello").unwrap();
            file.flush().unwrap();

            let handle = ReadWriteFileHandle::open(file.path().to_str().unwrap()).unwrap();
            let mapping = ReadWriteMmap::new(&handle, 10, 1).unwrap();

            assert_eq!(mapping.len(), 0);
            assert!(mapping.is_empty());
        }

        #[test]
        fn partial_mapping_at_end_of_file() {
            let mut file = NamedTempFile::new().unwrap();
            file.write_all(b"Hello, World!").unwrap();
            file.flush().unwrap();

            let handle = ReadWriteFileHandle::open(file.path().to_str().unwrap()).unwrap();
            let mapping = ReadWriteMmap::new(&handle, 10, 10).unwrap();

            assert_eq!(mapping.len(), 3);
            unsafe {
                let data = from_raw_parts_mut(mapping.data(), mapping.len());
                data.copy_from_slice(b"D!!");
            }

            let mut content = String::new();
            File::open(file.path())
                .unwrap()
                .read_to_string(&mut content)
                .unwrap();
            assert_eq!(content, "Hello, WorD!!");
        }
    }
}
