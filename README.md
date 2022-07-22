# bacon_rajan_cc

![Build Status](https://github.com/fitzgen/bacon-rajan-cc/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/bacon_rajan_cc.svg)](https://crates.io/crates/bacon_rajan_cc)
[![Documentation](https://docs.rs/bacon_rajan_cc/badge.svg)](https://docs.rs/bacon-rajan-cc)
[![Rust 1.34.2+](https://img.shields.io/badge/rust-1.34.2+-orange.svg)](https://www.rust-lang.org)

`Cc<T>`: A reference counted type with cycle collection for Rust. Concurrent or
stop-the-world. Based on the paper
["Concurrent Cycle Collection in Reference Counted Systems"][paper] by David
F. Bacon and V.T. Rajan. [JVM implementation](https://github.com/JikesRVM/JikesRVM/blob/8f6ac1854a73059595587b63fb4e8a3553bc7ff1/rvm/src/vm/memoryManagers/concurrent/VM_Allocator.java)

Currently only stop-the-world, not concurrent.

## Usage

Add to `Cargo.toml`:

Note this requires at least Rust 1.28 for the std::alloc api's.

```toml
[dependencies]
bacon_rajan_cc = "0.3"
```

Then, in your crate:

```rust
extern crate bacon_rajan_cc;
use bacon_rajan_cc::{Cc, Trace, Tracer};
```

## Documentation

[Read the docs!][docs]

[paper]: http://researcher.watson.ibm.com/researcher/files/us-bacon/Bacon01Concurrent.pdf
[docs]: https://docs.rs/bacon_rajan_cc/

## Alternatives
- https://github.com/jrmuizel/cc-mt (an experimental thread safe version of bacon-rajan-cc)
- https://github.com/withoutboats/shifgrethor
- https://github.com/Manishearth/rust-gc
- https://github.com/redradist/ferris-gc (a thread safe reimplementatin of rust-gc)
- https://github.com/Others/shredder
- https://github.com/jazz-lang/wafflelink (conservative on stack,precise on heap Immix Mark-Region GC with evacuation in Rust)
- https://github.com/artichoke/cactusref https://hyperbo.la/w/cactus-harvesting/
