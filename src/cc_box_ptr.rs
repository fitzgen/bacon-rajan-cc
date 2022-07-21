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
}

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
