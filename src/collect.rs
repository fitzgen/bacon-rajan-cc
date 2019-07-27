// Copyright 2015 The Rust Project Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::ptr::NonNull;
use std::cell::RefCell;

use cc_box_ptr::{CcBoxPtr, free};
use super::Color;

thread_local!(static ROOTS: RefCell<Vec<NonNull<CcBoxPtr>>> = RefCell::new(vec![]));

#[doc(hidden)]
pub fn add_root(box_ptr: NonNull<CcBoxPtr>) {
    ROOTS.with(|r| {
        let mut vec = r.borrow_mut();
        vec.push(box_ptr);
    });
}

/// Return the number of potential cycle roots currently buffered for cycle
/// collection.
///
/// Whenever a `Cc<T>`'s reference count is decremented, it has the possibility
/// of becoming the root of some cycle that is no longer live and can now be
/// reclaimed. These possible roots are buffered for cycle detection and
/// collection at a later point in time. This enables library users to avoid
/// frequent tracing and perform that tracing at a convenient time. Part of
/// choosing a convenient time might be when the number of potential cycle roots
/// reaches some critical threshold. This method allows you to check the current
/// number of possible roots buffered.
///
/// ```rust
/// use bacon_rajan_cc::{Cc, Trace, Tracer, number_of_roots_buffered};
/// use std::cell::RefCell;
///
/// struct Gadget {
///     parent: Option<Cc<RefCell<Gadget>>>,
///     children: Vec<Cc<RefCell<Gadget>>>,
///     // ...
/// }
///
/// impl Trace for Gadget {
///     fn trace(&mut self, _tracer: &mut Tracer) { /* ... */ }
/// }
///
/// fn add_child(parent: &mut Cc<RefCell<Gadget>>) -> Cc<RefCell<Gadget>> {
///     let child = Cc::new(RefCell::new(Gadget { parent: None, children: vec!() }));
///     child.borrow_mut().parent = Some(parent.clone());
///     parent.borrow_mut().children.push(child.clone());
///     child
/// }
///
/// pub fn main() {
///     // No possible roots, as we haven't created any `Cc<T>`s yet.
///     assert_eq!(number_of_roots_buffered(), 0);
///
///     {
///         let mut parent = Cc::new(RefCell::new(Gadget { parent: None, children: vec!() }));
///         let mut children = vec!();
///         for _ in 0..10 {
///             children.push(add_child(&mut parent));
///         }
///
///         // No possible roots, we have only incremented reference counts and
///         // created new `Cc<T>`s. We have not decremented any reference
///         // counts or created any dead cycles.
///         assert_eq!(number_of_roots_buffered(), 0);
///     }
///
///     // None of the Gadgets are reachable anymore, but they had cyclic
///     // references between parents and children. However, because their
///     // reference counts were decremented when we left the block, they should
///     // be buffered for cycle collection.
///     assert_eq!(number_of_roots_buffered(),
///                1 /* parent */ + 10 /* children */);
///
///     // If we had actually implemented `Trace` for `Gadget` rather than just
///     // stubbing it out, we could call `collect_cycles` here to reclaim the
///     // cycle.
/// }
/// ```
pub fn number_of_roots_buffered() -> usize {
    ROOTS.with(|r| r.borrow().len())
}

/// Invoke cycle collection for all `Cc<T>`s on this thread.
///
/// You may wish to do this when the roots buffer reaches a certain size, when
/// memory is low, or at opportune moments within your application (such as when
/// the user has been inactive for `n` seconds in a GUI application).
///
/// This happens in three phases:
///
/// 1. `mark_roots`: We mark the roots and decrement reference counts as we
/// go. This is optimistically removing the strong references held by the
/// potentially dead cycles.
///
/// 2. `scan_roots`: Then we perform a second traversal which marks the garbage
/// nodes with a reference count of 0 as White and the non-garbage nodes with a
/// reference count > 0 as Black. The latter group's reference count is restored
/// to its previous value from before step (1).
///
/// 3. `collect_roots`: Finally, the buffer of possible dead cycle roots is
/// emptied and members of dead cycles (White nodes) are dropped.
///
/// ```rust
/// use bacon_rajan_cc::{Cc, Trace, Tracer, collect_cycles};
/// use std::cell::RefCell;
///
/// // The number of Gadgets allocated at any given time.
/// thread_local!(static GADGET_COUNT: RefCell<usize> = RefCell::new(0));
///
/// struct Gadget {
///     parent: Option<Cc<RefCell<Gadget>>>,
///     children: Vec<Cc<RefCell<Gadget>>>,
///     // ...
/// }
///
/// impl Gadget {
///     fn new() -> Gadget {
///         GADGET_COUNT.with(|c| *c.borrow_mut() += 1);
///         Gadget { parent: None, children: vec!() }
///     }
/// }
///
/// impl Trace for Gadget {
///     fn trace(&mut self, tracer: &mut Tracer) {
///         self.parent.trace(tracer);
///         self.children.trace(tracer);
///     }
/// }
///
/// impl Drop for Gadget {
///     fn drop(&mut self) {
///         GADGET_COUNT.with(|c| *c.borrow_mut() -= 1);
///     }
/// }
///
/// fn add_child(parent: &mut Cc<RefCell<Gadget>>) -> Cc<RefCell<Gadget>> {
///     let child = Cc::new(RefCell::new(Gadget::new()));
///     child.borrow_mut().parent = Some(parent.clone());
///     parent.borrow_mut().children.push(child.clone());
///     child
/// }
///
/// pub fn main() {
///     // Initially, no gadgets.
///     GADGET_COUNT.with(|c| assert_eq!(*c.borrow(), 0));
///
///     {
///         // Create cycles.
///
///         let mut parent = Cc::new(RefCell::new(Gadget::new()));
///         for _ in 0..10 {
///             add_child(&mut parent);
///         }
///
///         // We created 1 parent and 10 child gadgets.
///         GADGET_COUNT.with(|c| assert_eq!(*c.borrow(), 11));
///     }
///
///     // The members of the cycle are now dead, but because of the cycles
///     // could not be eagerly collected.
///     GADGET_COUNT.with(|c| assert_eq!(*c.borrow(), 11));
///
///     // After calling `collect_cycles`, the cycles are detected and the
///     // members of the dead cycles are dropped.
///     collect_cycles();
///     GADGET_COUNT.with(|c| assert_eq!(*c.borrow(), 0));
/// }
/// ```
pub fn collect_cycles() {
    mark_roots();
    scan_roots();
    collect_roots();
}

/// Consider every node that's been stored in the buffer since the last
/// collection. If the node is Purple, then the last operation on it was a
/// decrement of its reference count, and it hasn't been touched since then. It
/// is potentially the root of a garbage cycle. Perform a graph traversal and
/// optimistically decrement reference counts as we go. At the end of the
/// traversal, anything whose reference count became 0 was part of a garbage
/// cycle. Anything whose reference count did not become 0 was not part of a
/// garbage cycle, and we will have to restore its old reference count in
/// `scan_roots`.
fn mark_roots() {
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

    let old_roots: Vec<_> = ROOTS.with(|r| {
        let mut v = r.borrow_mut();
        let drained = v.drain(..);
        drained.collect()
    });

    let mut new_roots : Vec<_> = old_roots.into_iter().filter_map(|mut s| {
        let keep = unsafe {
            let box_ptr : &mut CcBoxPtr = s.as_mut();
            if box_ptr.color() == Color::Purple {
                mark_gray(box_ptr);
                true
            } else {
                box_ptr.data().buffered.set(false);

                if box_ptr.color() == Color::Black && box_ptr.strong() == 0 {
                    free(s);
                }

                false
            }
        };

        if keep {
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

/// This is the second traversal, after marking. Color each node in the graph as
/// White nodes if its reference count is 0 and it is part of a garbage cycle,
/// or Black if the node is still live.
fn scan_roots() {
    fn scan_black(s: &mut CcBoxPtr) {
        s.data().color.set(Color::Black);
        s.trace(&mut |t| {
            t.inc_strong();
            if t.color() != Color::Black {
                scan_black(t);
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

    ROOTS.with(|r| {
        let mut v = r.borrow_mut();
        for s in &mut *v {
            let p : &mut CcBoxPtr = unsafe { s.as_mut() };
            scan(p);
        }
    });
}

/// Go through all the White roots and their garbage cycles and drop the nodes
/// as we go. If a White node is still in the roots buffer, then leave it
/// there. It will be freed in the nex collection when we iterate over the
/// buffer in `mark_roots`.
fn collect_roots() {
    fn collect_white(s: &mut (CcBoxPtr + 'static)) {
        if s.color() == Color::White && !s.buffered() {
            s.data().color.set(Color::Black);
            s.trace(&mut |t| {
                collect_white(t);
            });
            unsafe {
                free(s.into());
            }
        }
    }

    ROOTS.with(|r| {
        let mut v = r.borrow_mut();
        for mut s in v.drain(..) {
            let ptr : &mut CcBoxPtr = unsafe { s.as_mut() };
            ptr.data().buffered.set(false);
            collect_white(ptr);
        }
    });
}
