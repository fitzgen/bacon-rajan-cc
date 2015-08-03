// Copyright 2015 The Rust Project Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use super::{CcBoxData, Color};
use trace::Trace;

/// A trait to group all of the operations we need to be able to do on
/// `CcBox<T>`'s, potentially across different T types.
pub trait CcBoxPtr: Trace {
    /// Get this `CcBoxPtr`'s CcBoxData.
    fn data(&self) -> &CcBoxData;

    /// Get the color of this node.
    #[inline]
    fn color(&self) -> Color { self.data().color.get() }

    /// Return true if this node is in the buffer of possible cycle roots, false
    /// otherwise.
    #[inline]
    fn buffered(&self) -> bool { self.data().buffered.get() }

    /// Return the strong reference count.
    #[inline]
    fn strong(&self) -> usize { self.data().strong.get() }

    /// Increment this node's strong reference count.
    #[inline]
    fn inc_strong(&self) {
        self.data().strong.set(self.strong() + 1);
        self.data().color.set(Color::Black);
    }

    /// Decrement this node's strong reference count.
    #[inline]
    fn dec_strong(&self) { self.data().strong.set(self.strong() - 1); }

    /// Get this node's weak reference count, including the "strong weak"
    /// reference.
    #[inline]
    fn weak(&self) -> usize { self.data().weak.get() }

    /// Increment this node's weak reference count.
    #[inline]
    fn inc_weak(&self) { self.data().weak.set(self.weak() + 1); }

    /// Decrement this node's weak reference count.
    #[inline]
    fn dec_weak(&self) { self.data().weak.set(self.weak() - 1); }

    /// Run the Drop implementation for this node's value, but do not deallocate
    /// the box and its data, as there may still be live weak references that
    /// need to check the refcount on the box.
    unsafe fn drop_value(&mut self);

    /// Deallocate the box, assuming that the boxed value has already had its
    /// Drop implementation run.
    unsafe fn deallocate(&mut self);

    /// Drop the boxed value and deallocate the box if possible.
    unsafe fn free(&mut self) {
        debug_assert!(self.strong() == 0);
        debug_assert!(!self.buffered());

        // Remove the implicit "strong weak" pointer now that we've destroyed
        // the contents.
        self.dec_weak();

        self.drop_value();

        if self.weak() == 0 {
            self.deallocate();
        }
    }
}
