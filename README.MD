# lightweight-mmap

[![Crates.io](https://img.shields.io/crates/v/lightweight-mmap.svg)](https://crates.io/crates/lightweight-mmap)
[![Docs.rs](https://docs.rs/lightweight-mmap/badge.svg)](https://docs.rs/lightweight-mmap)
[![CI](https://github.com/Sewer56/lightweight-mmap/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/lightweight-mmap/actions)

## About

Simple memory mapping helpers for Rust, with minimal amount of code generated.

This crate provides the facilities for opening a file and mapping it to memory
with the minimal amount of code generated. 

## Motivation

This crate is for absolute freaks like me who wish to save 3-8KB of code size in their binaries;
with a thin limited wrapper providing complete zero overhead abstraction. Since `std` and 3rd party
mmap crates will compile and run a small amount of code which may not be needed for your use case.

If you have more advanced needs than present in this crate, consider using `std` for opening
file handles and a library like [memmap2-rs] and [mmap-rs] for mapping.

The API surface here is driven by [sewer56-archives-nx] and any other projects of mine which need mmap
in a tiny package.

## Characteristics

- Minimal code size overhead
- Platform-native file handles
- Read-only and read-write memory mappings
- Support for offset and length in mappings
- Supports zero sized mappings.
- Supports mappings to unaligned file offsets.
- Cross-platform compatibility
- Thread-safe (`Send` but not `Sync`)
- All opened handles can be accessed by multiple processes (Linux behaviour)

## Crate Features

- `std` (default): Enables standard library support
- `mmap` (default): Enables memory map operations, adding `mmap` cache info to handles on some platforms. Without this, library can only be used for opening raw file handles.
- `no-format`: Reduces binary size by skipping `core::fmt` formatting machinery as much as possible. Uses [itoa] and [nanokit] crates for minimal formatting.
- `trim-file-lengths`: Ensures memory maps cannot exceed file size by trimming mapping length. Adds a small overhead to map open time.

To use without standard library:
```toml
[dependencies]
lightweight-mmap = { version = "x.y.z", default-features = false }
```

To minimize binary size:
```toml
[dependencies]
lightweight-mmap = { version = "x.y.z", features = ["no-format"] }
```

Whether you should use `no-format` should depend on whether you already use `core::fmt` elsewhere in your
library/binary. Check with [cargo bloat]. If you don't, it's best to use `no-format` to reduce binary size.

## Platform Support

The crate is tested and supported on:

### Windows

- x86_64-pc-windows-msvc
- i686-pc-windows-msvc
- aarch64-pc-windows-msvc

### Linux

- x86_64-unknown-linux-gnu
- i686-unknown-linux-gnu
- aarch64-unknown-linux-gnu
- armv7-unknown-linux-gnueabihf

### macOS

- x86_64-apple-darwin
- aarch64-apple-darwin

### Android

- x86_64-linux-android
- i686-linux-android

--------

For other platforms, level of support is unknown.

## Examples

### File Handles

Open a read-only file handle to an existing file:
```rust,no_run
use lightweight_mmap::ReadOnlyFileHandle;
let handle = ReadOnlyFileHandle::open("Cargo.toml").unwrap();
```

Open a read-write file handle to an existing file:
```rust,no_run
use lightweight_mmap::ReadWriteFileHandle;
let handle = ReadWriteFileHandle::open("Cargo.toml").unwrap();
```

Create a new file with pre-allocated size:

```rust,no_run
use lightweight_mmap::ReadWriteFileHandle;
let handle = ReadWriteFileHandle::create_preallocated("test_file.txt", 1024).unwrap();
# std::fs::remove_file("test_file.txt").ok();
```

This will create a new file or overwrite an existing file.

### Memory Mapping

Create a read-only memory mapping:
```rust,no_run
# #[cfg(feature = "mmap")]
# {
use lightweight_mmap::{ReadOnlyFileHandle, ReadOnlyMmap};

// Open the file
let handle = ReadOnlyFileHandle::open("Cargo.toml").unwrap();

// Map 1024 bytes starting at offset 0
let mapping = ReadOnlyMmap::new(&handle, 0, 1024).unwrap();

// Access the mapped memory
let data = mapping.as_slice();
# }
```

Create a read-write memory mapping:

```rust,no_run
# #[cfg(feature = "mmap")]
# {
use lightweight_mmap::{ReadWriteFileHandle, ReadWriteMmap};

// Create a temporary file for writing
let handle = ReadWriteFileHandle::create_preallocated("temp_write.txt", 1024).unwrap();

// Map 1024 bytes starting at offset 0
let mut mapping = ReadWriteMmap::new(&handle, 0, 1024).unwrap();

// Access and modify the mapped memory
let data = mapping.as_mut_slice();
data[0] = 42;

# std::fs::remove_file("temp_write.txt").ok();
# }
```

Note: Memory mappings cannot outlive their file handles (compiler should ensure this), and the mapped
memory should be accessed carefully to avoid data races.

#### Use Across Threads

The default implementation of `Mmap` cannot be shared across threads, the lifetime
is tied to the current stack. If you need to share across threads, use the `Owned` variants.

```rust,no_run
# #[cfg(feature = "mmap")]
# {
use lightweight_mmap::{ReadOnlyFileHandle, OwnedReadOnlyMmap};
use std::sync::Arc;

let handle = Arc::new(ReadOnlyFileHandle::open("Cargo.toml").unwrap());
let mapping = unsafe { OwnedReadOnlyMmap::new(handle, 0, 1024).unwrap() };
# }
```

```rust,no_run
# #[cfg(feature = "mmap")]
# {
use lightweight_mmap::{ReadWriteFileHandle, OwnedReadWriteMmap};
use std::sync::Arc;

let handle = Arc::new(ReadWriteFileHandle::create_preallocated("temp_owned.txt", 1024).unwrap());
let mapping = unsafe { OwnedReadWriteMmap::new(handle, 0, 1024).unwrap() };
# std::fs::remove_file("temp_owned.txt").ok();
# }
```

The `Owned` variants internally use `Arc` to share the lifetime of the handle across
instances.

#### Memory Advice

Provide hints to the operating system about how memory mapped regions will be accessed:

```rust,no_run
# #[cfg(feature = "mmap")]
# {
use lightweight_mmap::{ReadOnlyFileHandle, ReadOnlyMmap, MemoryAdvice};

// Open and map the file
let handle = ReadOnlyFileHandle::open("Cargo.toml").unwrap();
let mapping = ReadOnlyMmap::new(&handle, 0, 1024).unwrap();

// Indicate we'll access this memory soon
mapping.advise(MemoryAdvice::WILL_NEED);

// Indicate sequential access pattern
mapping.advise(MemoryAdvice::SEQUENTIAL);

// Combine multiple hints
mapping.advise(MemoryAdvice::WILL_NEED | MemoryAdvice::SEQUENTIAL);
# }
```

Available advice flags:
- `WILL_NEED`: Indicates that the application expects to access the memory soon
- `SEQUENTIAL`: Indicates that memory access will be sequential from lower to higher addresses
- `RANDOM`: Indicates that memory access will be random (non-sequential)

Note: These are hints and may be ignored by the operating system.
Not all hints are supported on all platforms. On Windows, only `WILL_NEED` has an effect.

## API Differences: `std` vs `no_std`

The API surface changes depending on whether the `std` feature is enabled:

### With `std` feature (default)

When the `std` feature is enabled, file path parameters accept any type that implements `AsRef<std::path::Path>`:

```rust,no_run
# #[cfg(feature = "std")]
# {
use lightweight_mmap::{ReadOnlyFileHandle, ReadWriteFileHandle};
use std::path::Path;

// These all work with std feature enabled:
let handle1 = ReadOnlyFileHandle::open("Cargo.toml").unwrap();
let handle2 = ReadOnlyFileHandle::open(Path::new("Cargo.toml")).unwrap();
let handle3 = ReadWriteFileHandle::open("Cargo.toml").unwrap();
let handle4 = ReadWriteFileHandle::create_preallocated("temp_test.txt", 1024).unwrap();
# std::fs::remove_file("temp_test.txt").ok();
# }
```

### Without `std` feature (no_std)

When the `std` feature is disabled, file path parameters only accept `&str`:

```rust,no_run
# #[cfg(not(feature = "std"))]
# {
use lightweight_mmap::ReadOnlyFileHandle;

// Only string literals and &str work in no_std:
let handle1 = ReadOnlyFileHandle::open("Cargo.toml").unwrap();

// Path objects are not available in no_std environments
// Note: File creation via create_preallocated() typically requires std
# }
```

**Affected Methods:**

- `ReadOnlyFileHandle::open()`
- `ReadWriteFileHandle::open()`
- `ReadWriteFileHandle::create_preallocated()`

This design allows the library to work efficiently in both `std` and `no_std` environments while
providing the most ergonomic API for each context.

## Development

For information on how to work with this codebase, see [README-DEV.MD](README-DEV.MD).

## License

Licensed under [MIT](./LICENSE).  

[Learn more about Reloaded's general choice of licensing for projects.][reloaded-license].  

[codecov]: https://about.codecov.io/
[crates-io-key]: https://crates.io/settings/tokens
[nuget-key]: https://www.nuget.org/account/apikeys
[memmap2-rs]: https://github.com/RazrFalcon/memmap2-rs
[mmap-rs]: https://github.com/StephanvanSchaik/mmap-rs
[reloaded-license]: https://reloaded-project.github.io/Reloaded.MkDocsMaterial.Themes.R2/Pages/license/
[sewer56-archives-nx]: https://github.com/Sewer56/sewer56-archives-nx
[itoa]: https://crates.io/crates/itoa
[nanokit]: https://crates.io/crates/nanokit
[cargo bloat]: https://github.com/RazrFalcon/cargo-bloat