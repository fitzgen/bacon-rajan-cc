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

/// TODO FITZGEN
pub trait CcBoxPtr: Trace {
    /// TODO FITZGEN
    fn data(&self) -> &CcBoxData;

    /// TODO FITZGEN
    #[inline]
    fn color(&self) -> Color { self.data().color.get() }

    /// TODO FITZGEN
    #[inline]
    fn buffered(&self) -> bool { self.data().buffered.get() }

    /// TODO FITZGEN
    #[inline]
    fn strong(&self) -> usize { self.data().strong.get() }

    /// TODO FITZGEN
    #[inline]
    fn inc_strong(&self) {
        self.data().strong.set(self.strong() + 1);
        self.data().color.set(Color::Black);
    }

    /// TODO FITZGEN
    #[inline]
    fn dec_strong(&self) { self.data().strong.set(self.strong() - 1); }

    /// TODO FITZGEN
    #[inline]
    fn weak(&self) -> usize { self.data().weak.get() }

    /// TODO FITZGEN
    #[inline]
    fn inc_weak(&self) { self.data().weak.set(self.weak() + 1); }

    /// TODO FITZGEN
    #[inline]
    fn dec_weak(&self) { self.data().weak.set(self.weak() - 1); }

    /// TODO FITZGEN
    unsafe fn drop_value(&mut self);

    /// TODO FITZGEN
    unsafe fn deallocate(&mut self);

    /// TODO FITZGEN
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
