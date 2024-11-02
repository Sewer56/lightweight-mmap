#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(unix)]
pub mod unix;

#[cfg(unix)]
pub use unix::*;

use core::sync::atomic::{AtomicU32, Ordering};

/// Cached system allocation granularity.
static SYSTEM_ALLOCATION_GRANULARITY: AtomicU32 = AtomicU32::new(0);

/// Returns the system's memory allocation granularity.
///
/// This function caches the allocation granularity after the first call to avoid
/// expensive system calls on subsequent invocations.
///
/// # Returns
///
/// The system allocation granularity in bytes.
pub fn get_allocation_granularity() -> u32 {
    let cached = SYSTEM_ALLOCATION_GRANULARITY.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }

    let size = unsafe { query_allocation_granularity() };
    SYSTEM_ALLOCATION_GRANULARITY.store(size, Ordering::Relaxed);
    size
}
