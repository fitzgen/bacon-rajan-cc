# bacon_rajan_cc

[![Build Status](https://travis-ci.org/fitzgen/bacon-rajan-cc.png?branch=master)](https://travis-ci.org/fitzgen/bacon-rajan-cc)

[![crates.io](http://meritbadge.herokuapp.com/bacon_rajan_cc)](https://crates.io/crates/bacon_rajan_cc)

A reference counted type with cycle collection for Rust. Concurrent or
stop-the-world. Based on the paper
["Concurrent Cycle Collection in Reference Counted Systems"][paper] by David
F. Bacon and V.T. Rajan.

**Very much a work-in-progress! Currently only stop-the-world.**

[paper]: http://researcher.watson.ibm.com/researcher/files/us-bacon/Bacon01Concurrent.pdf
