#![doc = include_str!("../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "c-exports")]
pub mod exports;

pub mod handles;
#[cfg(feature = "mmap")]
pub mod mmap;
pub(crate) mod util;

extern crate alloc;
