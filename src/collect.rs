// Copyright 2015 The Rust Project Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use super::{CcBoxPtr, Color};

use core::nonzero::NonZero;
use std::cell::RefCell;

thread_local!(static ROOTS: RefCell<Vec<NonZero<*mut CcBoxPtr>>> = RefCell::new(vec![]));

#[doc(hidden)]
pub fn add_root(box_ptr: NonZero<*mut CcBoxPtr>) {
    ROOTS.with(|r| {
        let mut vec = r.borrow_mut();
        vec.push(box_ptr);
    });
}

/// TODO FITZGEN
pub fn collect_cycles() {
    mark_roots();
    scan_roots();
    collect_roots();
}

/// TODO FITZGEN
fn mark_roots() {
    let roots: Vec<_> = ROOTS.with(|r| {
        let mut v = r.borrow_mut();
        let drained = v.drain(..);
        drained.collect()
    });

    let mut new_roots : Vec<_> = roots.into_iter().filter_map(|s| {
        let keep_root = unsafe {
            let box_ptr : &mut CcBoxPtr = &mut **s;
            if box_ptr.color() == Color::Purple {
                mark_gray(box_ptr);
                true
            } else {
                box_ptr.data().buffered.set(false);

                if box_ptr.color() == Color::Black && box_ptr.strong() == 0 {
                    box_ptr.free();
                }

                false
            }
        };

        if keep_root {
            Some(s)
        } else {
            None
        }
    }).collect();

    ROOTS.with(|r| {
        let mut v = r.borrow_mut();
        v.append(&mut new_roots);
    });
}

/// TODO FITZGEN
fn mark_gray(cc_box_ptr: &mut CcBoxPtr) {
    if cc_box_ptr.color() == Color::Gray {
        return;
    }

    cc_box_ptr.data().color.set(Color::Gray);

    cc_box_ptr.trace(&mut |t| {
        t.dec_strong();
        mark_gray(t);
    });
}

/// TODO FITZGEN
fn scan_roots() {
    ROOTS.with(|r| {
        let v = r.borrow();
        for s in &*v {
            let p : &mut CcBoxPtr = unsafe { &mut ***s };
            scan(p);
        }
    });
}

fn scan(s: &mut CcBoxPtr) {
    if s.color() != Color::Gray {
        return;
    }

    if s.strong() > 0 {
        scan_black(s);
    } else {
        s.data().color.set(Color::White);
        s.trace(&mut |t| {
            scan(t);
        });
    }
}

fn scan_black(s: &mut CcBoxPtr) {
    s.data().color.set(Color::Black);
    s.trace(&mut |t| {
        t.inc_strong();
        if t.color() != Color::Black {
            scan_black(t);
        }
    });
}

/// TODO FITZGEN
fn collect_roots() {
    ROOTS.with(|r| {
        let mut v = r.borrow_mut();
        for s in v.drain(..) {
            let ptr : &mut CcBoxPtr = unsafe { &mut **s };
            ptr.data().buffered.set(false);
            collect_white(ptr);
        }
    });
}

fn collect_white(s: &mut CcBoxPtr) {
    if s.color() == Color::White && !s.buffered() {
        s.data().color.set(Color::Black);
        s.trace(&mut |t| {
            collect_white(t);
        });
        unsafe {
            s.free();
        }
    }
}
