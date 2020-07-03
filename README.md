# bacon_rajan_cc

[![Build Status](https://travis-ci.org/fitzgen/bacon-rajan-cc.png?branch=master)](https://travis-ci.org/fitzgen/bacon-rajan-cc)

[![crates.io](http://meritbadge.herokuapp.com/bacon_rajan_cc)](https://crates.io/crates/bacon_rajan_cc)

`Cc<T>`: A reference counted type with cycle collection for Rust. Concurrent or
stop-the-world. Based on the paper
["Concurrent Cycle Collection in Reference Counted Systems"][paper] by David
F. Bacon and V.T. Rajan.

Currently only stop-the-world, not concurrent.

## Usage

Add to `Cargo.toml`:

Note this requires at least Rust 1.28 for the std::alloc api's.

```toml
[dependencies]
bacon_rajan_cc = "0.2"
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
https://github.com/withoutboats/shifgrethor
https://github.com/Manishearth/rust-gc
https://github.com/Others/shredder
