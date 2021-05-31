# bacon_rajan_cc

[![Build Status](https://travis-ci.org/fitzgen/bacon-rajan-cc.png?branch=master)](https://travis-ci.org/fitzgen/bacon-rajan-cc)

[![crates.io](http://meritbadge.herokuapp.com/bacon_rajan_cc)](https://crates.io/crates/bacon_rajan_cc)

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
