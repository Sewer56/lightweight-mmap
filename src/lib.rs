#![doc = include_str!("../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]
// #[cfg(feature = "c-exports")]
// pub mod exports;

pub mod handles;
#[cfg(feature = "mmap")]
pub mod mmap;
pub(crate) mod util;

extern crate alloc;

// Re-export the main types at the crate root for convenience
pub use handles::{HandleOpenError, ReadOnlyFileHandle, ReadWriteFileHandle};
#[cfg(feature = "mmap")]
pub use mmap::{
    MemoryAdvice, MmapError, OwnedReadOnlyMmap, OwnedReadWriteMmap, ReadOnlyMmap, ReadWriteMmap,
};
