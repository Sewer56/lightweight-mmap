use libc::*;

/// Query the system allocation granularity on Unix systems, which is just the page size.
///
/// # Safety
///
/// This function is unsafe because it calls the raw sysconf syscall.
pub unsafe fn query_allocation_granularity() -> u32 {
    sysconf(_SC_PAGESIZE) as u32
}
