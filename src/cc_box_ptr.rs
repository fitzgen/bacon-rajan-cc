// Copyright 2015 The Rust Project Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::ptr::NonNull;

use crate::trace::Trace;
use crate::CcBoxData;

/// A trait to group all of the operations we need to be able to do on
/// `CcBox<T>`'s, potentially across different T types.
pub trait CcBoxPtr: Trace {
    /// Get this `CcBoxPtr`'s CcBoxData.
    fn data(&self) -> &CcBoxData;
    // Get a mutable reference the value inside this `CcBoxPtr`.
    // We use this for calling Drop on the value instead of calling
    // it on the `CcBoxPtr` directly, because we want to avoid holding
    // a mutable reference to the entire type implementing `CcBoxPtr` 
    // because we may need to access the data during a drop if there's
    // a self cycle.
    fn value(&mut self) -> &mut dyn Dropable;
}

// An empty trait object that we can use to call
// ptr::drop_in_place on.
pub trait Dropable {
}
// Implemented for everything
impl<T> Dropable for T {}

/// Deallocate the box if possible. `s` should already have been dropped.
pub unsafe fn free(s: NonNull<dyn CcBoxPtr>) {
    debug_assert!(s.as_ref().data().strong() == 0);
    debug_assert!(!s.as_ref().data().buffered());

    // Remove the implicit "strong weak" pointer now that we've destroyed
    // the contents.
    s.as_ref().data().dec_weak();

    if s.as_ref().data().weak() == 0 {
        crate::deallocate(s);
    }
}
