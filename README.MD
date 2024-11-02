# lightweight-mmap

[![Crates.io](https://img.shields.io/crates/v/lightweight-mmap.svg)](https://crates.io/crates/lightweight-mmap)
[![Docs.rs](https://docs.rs/lightweight-mmap/badge.svg)](https://docs.rs/lightweight-mmap)
[![CI](https://github.com/Sewer56/lightweight-mmap/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/lightweight-mmap/actions)

## About

Simple memory mapping helpers for Rust, with minimal amount of code generated.

This crate provides the facilities for opening a file and mapping it to memory
with the minimal amount of code generated. Use this if you're a fellow freak trying to
minimize code size.

For a more feature rich library, see [memmap2-rs] and [mmap-rs].

## Examples

### Open a Read Only File Handle

```rust
let handle = ReadOnlyFileHandle::open("assets/test_file.txt").unwrap();
```

### Open a Read Write File Handle

```rust
let handle = ReadWriteFileHandle::open("assets/test_file.txt").unwrap();
```

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