pub mod error;
pub use error::*;

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
/// **Note:** `ReadOnlyFileHandle` is `Send` but not `Sync`. It should only be
/// accessed from a single thread.
pub struct ReadOnlyFileHandle {
    inner: InnerHandle,
}

unsafe impl Send for ReadOnlyFileHandle {}

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn can_open_read_only_file_handle() {
        let path = Path::new("assets/test_file.txt");
        let path_str = path.to_str().unwrap();
        let handle = ReadOnlyFileHandle::open("assets/test_file.txt").unwrap();

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
        let path = Path::new("assets/test_file.txt");
        let path_str = path.to_str().unwrap();

        let handle1 = ReadOnlyFileHandle::open(path_str).unwrap();
        let handle2 = ReadOnlyFileHandle::open(path_str).unwrap();

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
}
