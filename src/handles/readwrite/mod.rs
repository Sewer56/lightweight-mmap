pub use super::*;

/// Platform-specific implementations
#[cfg(target_os = "windows")]
mod windows;

#[cfg(unix)]
mod unix;

/// Platform-specific inner handle.
#[cfg(target_os = "windows")]
use windows::*;

#[cfg(unix)]
use unix::*;

/// A read-write file handle that allows shared access to the file.
///
/// This struct provides a platform-independent way to open a file in read-write
/// mode while allowing other processes to access the file simultaneously.
///
/// **Note:** [`ReadWriteFileHandle`] is [`Send`] but not [`Sync`]. It should only be
/// accessed from a single thread.
pub struct ReadWriteFileHandle {
    inner: InnerHandle,
}

impl ReadWriteFileHandle {
    /// Opens a file in read-write mode with shared access.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to open.
    ///
    /// # Errors
    ///
    /// Returns a `HandleOpenError` if the file cannot be opened.
    pub fn open(path: &str) -> Result<Self, HandleOpenError> {
        let inner = InnerHandle::open(path)?;
        Ok(ReadWriteFileHandle { inner })
    }

    /// Creates a new file with pre-allocated size.
    ///
    /// This creates a new file and pre-allocates the specified amount of space.
    /// If the file already exists, it will be truncated and pre-allocated to the specified size.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the file should be created
    /// * `size` - The size to pre-allocate in bytes
    ///
    /// # Errors
    ///
    /// Returns a [`HandleOpenError`] if:
    /// - The file cannot be created
    /// - Pre-allocation fails
    /// - The path is invalid
    pub fn create_preallocated(path: &str, size: i64) -> Result<Self, HandleOpenError> {
        let inner = InnerHandle::create_preallocated(path, size)?;
        Ok(ReadWriteFileHandle { inner })
    }

    /// Returns a reference to the underlying file descriptor or handle.
    ///
    /// This can be used for further operations if needed.
    pub fn handle(&self) -> &InnerHandle {
        &self.inner
    }

    /// Returns the size of the file in bytes.
    pub fn size(&self) -> Result<i64, HandleOpenError> {
        #[cfg(unix)]
        {
            unix_common::get_file_size(self.inner.fd())
        }

        #[cfg(target_os = "windows")]
        {
            windows_common::get_file_size(self.inner.handle())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::*, io::*};
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn can_open_read_write_file_handle() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();
        let handle = ReadWriteFileHandle::open(path).unwrap();

        #[cfg(unix)]
        {
            assert!(handle.handle().fd() >= 0);
        }

        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            assert!(handle.handle().handle() != INVALID_HANDLE_VALUE);
        }
    }

    #[test]
    fn can_open_handle_multiple_times() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();
        let handle1 = ReadWriteFileHandle::open(path).unwrap();
        let handle2 = ReadWriteFileHandle::open(path).unwrap();

        #[cfg(unix)]
        {
            assert!(handle1.handle().fd() >= 0);
            assert!(handle2.handle().fd() >= 0);
        }

        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            assert!(handle1.handle().handle() != INVALID_HANDLE_VALUE);
            assert!(handle2.handle().handle() != INVALID_HANDLE_VALUE);
        }
    }

    #[test]
    fn can_create_preallocated_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("preallocated.bin");
        let path_str = path.to_str().unwrap();

        let expected_size = 65536;
        let handle = ReadWriteFileHandle::create_preallocated(path_str, expected_size).unwrap();

        // Verify handle is valid
        #[cfg(unix)]
        {
            assert!(handle.handle().fd() >= 0);
        }
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            assert!(handle.handle().handle() != INVALID_HANDLE_VALUE);
        }

        // Verify file size
        let actual_size = handle.size().unwrap();
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn preallocated_file_is_writable() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("writable.bin");
        let path_str = path.to_str().unwrap();

        let expected_size = 1024;
        let handle = ReadWriteFileHandle::create_preallocated(path_str, expected_size).unwrap();

        // Write some data using regular file operations
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path_str)
            .unwrap();
        file.write_all(b"Hello, World!").unwrap();
        file.flush().unwrap();

        // Verify file size hasn't changed
        let actual_size = handle.size().unwrap();
        assert_eq!(actual_size, expected_size);

        // Verify content was written
        let mut contents = String::new();
        File::open(path_str)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        assert!(contents.starts_with("Hello, World!"));
    }

    #[test]
    fn preallocated_truncates_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("truncate.bin");
        let path_str = path.to_str().unwrap();

        // Create file with initial content
        let mut file = File::create(path_str).unwrap();
        file.write_all(b"Hello World!!").unwrap();
        file.flush().unwrap();

        // Create preallocated handle
        let expected_size = 5;
        let handle = ReadWriteFileHandle::create_preallocated(path_str, expected_size).unwrap();

        // Verify file was truncated to new size
        let actual_size = handle.size().unwrap();
        assert_eq!(actual_size, expected_size);

        // Verify old content is gone
        let mut buffer = Vec::new();
        File::open(path_str)
            .unwrap()
            .read_to_end(&mut buffer)
            .unwrap();
        //println!("Raw bytes after truncating: {:?}", buffer);
        assert_eq!(buffer, b"Hello");
    }

    #[test]
    fn preallocated_extends_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("truncate.bin");
        let path_str = path.to_str().unwrap();

        // Create file with initial content
        let mut file = File::create(path_str).unwrap();
        file.write_all(b"Hello").unwrap();
        file.flush().unwrap();

        // Create preallocated handle
        let expected_size = 10;
        let handle = ReadWriteFileHandle::create_preallocated(path_str, expected_size).unwrap();

        // Verify file was extended to new size
        let actual_size = handle.size().unwrap();
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn preallocated_fails_on_invalid_path() {
        let result = ReadWriteFileHandle::create_preallocated(
            "/path/that/definitely/does/not/exist/file.bin",
            1024,
        );
        assert!(result.is_err());
    }

    #[test]
    #[cfg(feature = "mmap")]
    fn can_memory_map_preallocated_file() {
        use crate::mmap::ReadWriteMmap;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("mappable.bin");
        let path_str = path.to_str().unwrap();

        let expected_size = 65536; // 1MB
        let handle = ReadWriteFileHandle::create_preallocated(path_str, expected_size).unwrap();

        // Try to create a memory mapping
        let mapping = ReadWriteMmap::new(&handle, 0, expected_size as usize).unwrap();

        assert_eq!(mapping.len(), expected_size.try_into().unwrap());
        assert!(!mapping.is_empty());
    }
}
