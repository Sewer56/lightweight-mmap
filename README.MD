# lightweight-mmap

[![Crates.io](https://img.shields.io/crates/v/lightweight-mmap.svg)](https://crates.io/crates/lightweight-mmap)
[![Docs.rs](https://docs.rs/lightweight-mmap/badge.svg)](https://docs.rs/lightweight-mmap)
[![CI](https://github.com/Sewer56/lightweight-mmap/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/lightweight-mmap/actions)

Simple memory mapping helpers for Rust, with minimal amount of code generated.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
lightweight-mmap = "0.7"
```

```rust
use lightweight_mmap::ReadOnlyFileHandle;
let handle = ReadOnlyFileHandle::open("Cargo.toml").unwrap();
```

See the [lightweight-mmap crate documentation](./src/lightweight-mmap/README.MD) for detailed usage.

## Crates

- [lightweight-mmap](./src/lightweight-mmap/README.MD): Memory mapping helpers with minimal code size

## Developer Manual

For step-by-step development guidance, see the [Developer Manual](https://reloaded-project.github.io/reloaded-templates-rust/manual/).

## Contributing

We welcome contributions! See the [Contributing Guide](https://reloaded-project.github.io/reloaded-templates-rust/manual/#contributing) for details.

## License

Licensed under [MIT](./LICENSE).