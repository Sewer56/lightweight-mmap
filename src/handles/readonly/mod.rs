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

/// A read-only file handle that allows shared access to the file.
///
/// This struct provides a platform-independent way to open a file in read-only
/// mode while allowing other processes to access the file simultaneously.
///
/// **Note:** [`ReadOnlyFileHandle`] is [`Send`] but not [`Sync`]. It should only be
/// accessed from a single thread.
pub struct ReadOnlyFileHandle {
    inner: InnerHandle,
}

impl ReadOnlyFileHandle {
    /// Opens a file in read-only mode with shared access.
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
        Ok(ReadOnlyFileHandle { inner })
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
    use std::{fs::OpenOptions, io::Write};

    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn can_open_read_only_file_handle() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();
        let handle = ReadOnlyFileHandle::open(path).unwrap();

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
        let handle1 = ReadOnlyFileHandle::open(path).unwrap();
        let handle2 = ReadOnlyFileHandle::open(path).unwrap();

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
    fn size_fails_on_nonexistent_file() {
        let handle_result = ReadOnlyFileHandle::open("nonexistent_file_test.txt");
        assert!(handle_result.is_err());
    }

    #[test]
    fn can_get_size_of_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();
        let handle = ReadOnlyFileHandle::open(path).unwrap();

        let size = handle.size().unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn can_get_size_of_nonempty_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        // Write some test data
        {
            let mut file = OpenOptions::new().write(true).open(path).unwrap();
            file.write_all(b"Hello, World!").unwrap();
            file.flush().unwrap();
        }

        let handle = ReadOnlyFileHandle::open(path).unwrap();
        let size = handle.size().unwrap();
        assert_eq!(size, 13); // Length of "Hello, World!"
    }
}
