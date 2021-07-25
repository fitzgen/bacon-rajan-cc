// Copyright 2015 The Rust Project Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Thread-local reference-counted boxes (the `Cc<T>` type).
//!
//! The `Cc<T>` type provides shared ownership of an immutable value.
//! Destruction is deterministic, and will occur as soon as the last owner is
//! gone. It is marked as non-sendable because it avoids the overhead of atomic
//! reference counting.
//!
//! The `downgrade` method can be used to create a non-owning `Weak<T>` pointer
//! to the box. A `Weak<T>` pointer can be upgraded to an `Cc<T>` pointer, but
//! will return `None` if the value has already been dropped.
//!
//! For example, a tree with parent pointers can be represented by putting the
//! nodes behind strong `Cc<T>` pointers, and then storing the parent pointers
//! as `Weak<T>` pointers.
//!
//! # Examples
//!
//! Consider a scenario where a set of `Gadget`s are owned by a given `Owner`.
//! We want to have our `Gadget`s point to their `Owner`. We can't do this with
//! unique ownership, because more than one gadget may belong to the same
//! `Owner`. `Cc<T>` allows us to share an `Owner` between multiple `Gadget`s,
//! and have the `Owner` remain allocated as long as any `Gadget` points at it.
//!
//! ```rust
//! use bacon_rajan_cc::{Cc, Trace, Tracer, collect_cycles};
//!
//! struct Owner {
//!     name: String
//!     // ...other fields
//! }
//!
//! impl Trace for Owner {
//!     // Note: nothing to trace since `Owner` doesn't own any Cc<T> things.
//!     fn trace(&self, _tracer: &mut Tracer) { }
//! }
//!
//! struct Gadget {
//!     id: i32,
//!     owner: Cc<Owner>
//!     // ...other fields
//! }
//!
//! fn main() {
//!     // Create a reference counted Owner.
//!     let gadget_owner : Cc<Owner> = Cc::new(
//!             Owner { name: String::from("Gadget Man") }
//!     );
//!
//!     // Create Gadgets belonging to gadget_owner. To increment the reference
//!     // count we clone the `Cc<T>` object.
//!     let gadget1 = Gadget { id: 1, owner: gadget_owner.clone() };
//!     let gadget2 = Gadget { id: 2, owner: gadget_owner.clone() };
//!
//!     drop(gadget_owner);
//!
//!     // Despite dropping gadget_owner, we're still able to print out the name
//!     // of the Owner of the Gadgets. This is because we've only dropped the
//!     // reference count object, not the Owner it wraps. As long as there are
//!     // other `Cc<T>` objects pointing at the same Owner, it will remain
//!     // allocated. Notice that the `Cc<T>` wrapper around Gadget.owner gets
//!     // automatically dereferenced for us.
//!     println!("Gadget {} owned by {}", gadget1.id, gadget1.owner.name);
//!     println!("Gadget {} owned by {}", gadget2.id, gadget2.owner.name);
//!
//!     // At the end of the method, gadget1 and gadget2 get destroyed, and with
//!     // them the last counted references to our Owner. Gadget Man now gets
//!     // destroyed as well.
//!     drop(gadget1);
//!     drop(gadget2);
//!     collect_cycles()
//! }
//! ```
//!
//! If our requirements change, and we also need to be able to traverse from
//! Owner → Gadget, we will run into problems: an `Cc<T>` pointer from Owner
//! → Gadget introduces a cycle between the objects. This means that their
//! reference counts can never reach 0, and the objects will remain allocated: a
//! memory leak. In order to get around this, we can use `Weak<T>` pointers.
//! These pointers don't contribute to the total count.
//!
//! Rust actually makes it somewhat difficult to produce this loop in the first
//! place: in order to end up with two objects that point at each other, one of
//! them needs to be mutable. This is problematic because `Cc<T>` enforces
//! memory safety by only giving out shared references to the object it wraps,
//! and these don't allow direct mutation. We need to wrap the part of the
//! object we wish to mutate in a `RefCell`, which provides *interior
//! mutability*: a method to achieve mutability through a shared reference.
//! `RefCell` enforces Rust's borrowing rules at runtime.  Read the `Cell`
//! documentation for more details on interior mutability.
//!
//! ```rust
//! use bacon_rajan_cc::{Cc, Weak, Trace, Tracer, collect_cycles};
//! use std::cell::RefCell;
//!
//! struct Owner {
//!     name: String,
//!     gadgets: RefCell<Vec<Weak<Gadget>>>
//!     // ...other fields
//! }
//!
//! impl Trace for Owner {
//!     fn trace(&self, _tracer: &mut Tracer) { }
//! }
//!
//! struct Gadget {
//!     id: i32,
//!     owner: Cc<Owner>
//!     // ...other fields
//! }
//!
//! impl Trace for Gadget {
//!     fn trace(&self, tracer: &mut Tracer) {
//!         tracer(&self.owner);
//!     }
//! }
//!
//! fn main() {
//!     // Create a reference counted Owner. Note the fact that we've put the
//!     // Owner's vector of Gadgets inside a RefCell so that we can mutate it
//!     // through a shared reference.
//!     let gadget_owner : Cc<Owner> = Cc::new(
//!             Owner {
//!                 name: "Gadget Man".to_string(),
//!                 gadgets: RefCell::new(Vec::new())
//!             }
//!     );
//!
//!     // Create Gadgets belonging to gadget_owner as before.
//!     let gadget1 = Cc::new(Gadget{id: 1, owner: gadget_owner.clone()});
//!     let gadget2 = Cc::new(Gadget{id: 2, owner: gadget_owner.clone()});
//!
//!     // Add the Gadgets to their Owner. To do this we mutably borrow from
//!     // the RefCell holding the Owner's Gadgets.
//!     gadget_owner.gadgets.borrow_mut().push(gadget1.clone().downgrade());
//!     gadget_owner.gadgets.borrow_mut().push(gadget2.clone().downgrade());
//!
//!     // Iterate over our Gadgets, printing their details out
//!     for gadget_opt in gadget_owner.gadgets.borrow().iter() {
//!
//!         // gadget_opt is a Weak<Gadget>. Since weak pointers can't guarantee
//!         // that their object is still allocated, we need to call upgrade()
//!         // on them to turn them into a strong reference. This returns an
//!         // Option, which contains a reference to our object if it still
//!         // exists.
//!         let gadget = gadget_opt.upgrade().unwrap();
//!         println!("Gadget {} owned by {}", gadget.id, gadget.owner.name);
//!     }
//!
//!     // At the end of the method, gadget_owner, gadget1 and gadget2 get
//!     // destroyed. There are now no strong (`Cc<T>`) references to the gadgets.
//!     // Once they get destroyed, the Gadgets get destroyed. This zeroes the
//!     // reference count on Gadget Man, so he gets destroyed as well.
//!     drop((gadget_owner, gadget1, gadget2));
//!     collect_cycles();
//! }
//! ```

#![deny(missing_docs)]

extern crate core;
use core::cell::Cell;
use core::clone::Clone;
use core::cmp::{PartialEq, PartialOrd, Eq, Ord, Ordering};
use core::default::Default;
use core::fmt;
use core::hash::{Hasher, Hash};
use core::mem::forget;
use std::ptr::NonNull;
use core::ops::{Deref, Drop};
use core::option::Option;
use core::option::Option::{Some, None};
use core::ptr;
use core::result::Result;
use core::result::Result::{Ok, Err};

use std::alloc::{dealloc, Layout};

/// Tracing traits, types, and implementation.
pub mod trace;
pub use trace::{Trace, Tracer};

/// Implementation of cycle detection and collection.
pub mod collect;
pub use collect::{collect_cycles, number_of_roots_buffered};

mod cc_box_ptr;
use cc_box_ptr::CcBoxPtr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[doc(hidden)]
pub enum Color {
    /// In use or free.
    Black,

    /// Possible member of a cycle.
    Gray,

    /// Member of a garbage cycle.
    White,

    /// Possible root of cycle.
    Purple,

    /// Candidate cycle undergoing sigma-computation. Not yet in use.
    #[allow(dead_code)]
    Red,

    /// Candidate cycle awaiting epoch boundary. Not yet in use.
    #[allow(dead_code)]
    Orange,
}

#[derive(Debug)]
#[doc(hidden)]
pub struct CcBoxData {
    strong: Cell<usize>,
    weak: Cell<usize>,
    buffered: Cell<bool>,
    color: Cell<Color>,
}

#[derive(Debug)]
struct CcBox<T: Trace> {
    value: T,
    data: CcBoxData,
}

/// A reference-counted pointer type over an immutable value.
///
/// See the [module level documentation](./) for more details.
pub struct Cc<T: 'static + Trace> {
    // FIXME #12808: strange names to try to avoid interfering with field
    // accesses of the contained type via Deref
    _ptr: NonNull<CcBox<T>>,
}

impl<T: Trace> Cc<T> {
    /// Constructs a new `Cc<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let five = Cc::new(5);
    /// ```
    pub fn new(value: T) -> Cc<T> {
        unsafe {
            Cc {
                // There is an implicit weak pointer owned by all the strong
                // pointers, which ensures that the weak destructor never frees
                // the allocation while the strong destructor is running, even
                // if the weak pointer is stored inside the strong one.
                _ptr: NonNull::new_unchecked(Box::into_raw(Box::new(CcBox {
                    value: value,
                    data: CcBoxData {
                        strong: Cell::new(1),
                        weak: Cell::new(1),
                        buffered: Cell::new(false),
                        color: Cell::new(Color::Black),
                    }
                }))),
            }
        }
    }

    /// Downgrades the `Cc<T>` to a `Weak<T>` reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let five = Cc::new(5);
    ///
    /// let weak_five = five.downgrade();
    /// ```
    pub fn downgrade(&self) -> Weak<T> {
        self.inc_weak();
        Weak { _ptr: self._ptr }
    }
}

impl<T: Trace> Cc<T> {
    unsafe fn release(&mut self) {
        debug_assert!(self.strong() == 0);

        crate::drop_value(self._ptr);

        self.data().color.set(Color::Black);

        // If it is in the buffer, then it will be freed later in the
        // `mark_roots` procedure.
        if self.buffered() {
            return;
        }

        crate::cc_box_ptr::free(self._ptr);
    }

    fn possible_root(&mut self) {
        debug_assert!(self.strong() > 0);

        if self.color() == Color::Purple {
            return;
        }

        self.data().color.set(Color::Purple);
        if self.buffered() {
            return;
        }

        self.data().buffered.set(true);
        let ptr : NonNull<dyn CcBoxPtr> = self._ptr;
        collect::add_root(ptr);
    }
}

impl<T: 'static + Trace> Cc<T> {
    /// Returns true if there are no other `Cc` or `Weak<T>` values that share
    /// the same inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc;
    /// use bacon_rajan_cc::{Cc, collect_cycles};
    /// {
    ///   let five = Cc::new(5);
    ///   assert_eq!(five.is_unique(), true);
    ///
    ///   let another_five = five.clone();
    ///   assert_eq!(five.is_unique(), false);
    ///   assert_eq!(another_five.is_unique(), false);
    /// }
    /// collect_cycles();
    /// ```
    #[inline]
    pub fn is_unique(&self) -> bool {
        self.weak_count() == 0 && self.strong_count() == 1
    }

    /// Unwraps the contained value if the `Cc<T>` is unique.
    ///
    /// If the `Cc<T>` is not unique, an `Err` is returned with the same `Cc<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::{Cc, collect_cycles};
    /// {
    ///   let x = Cc::new(3);
    ///   assert_eq!(x.try_unwrap(), Ok(3));
    ///
    ///   let x = Cc::new(4);
    ///   let _y = x.clone();
    ///   assert_eq!(x.try_unwrap(), Err(Cc::new(4)));
    /// }
    /// collect_cycles();
    /// ```
    #[inline]
    pub fn try_unwrap(self) -> Result<T, Cc<T>> {
        if self.is_unique() {
            unsafe {
                // Copy the contained object.
                let val = ptr::read(&*self);
                // Destruct the box and skip our Drop. We can ignore the
                // refcounts because we know we're unique.
                dealloc(self._ptr.cast().as_ptr(), Layout::new::<CcBox<T>>());
                forget(self);
                Ok(val)
            }
        } else {
            Err(self)
        }
    }

    /// Returns a mutable reference to the contained value if the `Cc<T>` is
    /// unique.
    ///
    /// Returns `None` if the `Cc<T>` is not unique.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::{Cc, collect_cycles};
    /// {
    ///   let mut x = Cc::new(3);
    ///   *Cc::get_mut(&mut x).unwrap() = 4;
    ///   assert_eq!(*x, 4);
    ///
    ///   let _y = x.clone();
    ///   assert!(Cc::get_mut(&mut x).is_none());
    /// }
    /// collect_cycles();
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_unique() {
            let inner = unsafe { self._ptr.as_mut() };
            Some(&mut inner.value)
        } else {
            None
        }
    }

    /// Get the number of strong references to this value.
    #[inline]
    pub fn strong_count(&self) -> usize { self.strong() }

    /// Get the number of weak references to this value.
    #[inline]
    pub fn weak_count(&self) -> usize { self.weak() - 1 }
}

impl<T: 'static + Clone + Trace> Cc<T> {
    /// Make a mutable reference from the given `Cc<T>`.
    ///
    /// This is also referred to as a copy-on-write operation because the inner
    /// data is cloned if the reference count is greater than one.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let mut five = Cc::new(5);
    ///
    /// let mut_five = five.make_unique();
    /// ```
    #[inline]
    pub fn make_unique(&mut self) -> &mut T {
        if !self.is_unique() {
            *self = Cc::new((**self).clone())
        }
        // This unsafety is ok because we're guaranteed that the pointer
        // returned is the *only* pointer that will ever be returned to T. Our
        // reference count is guaranteed to be 1 at this point, and we required
        // the `Cc<T>` itself to be `mut`, so we're returning the only possible
        // reference to the inner value.
        let inner = unsafe { self._ptr.as_mut() };
        &mut inner.value
    }
}

impl<T: Trace> Cc<T> {
    // Returns `true` if the two `Cc`s point to the same allocation
    /// (in a vein similar to [`ptr::eq`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::{Cc, collect_cycles};
    /// {
    ///    let five = Cc::new(5);
    ///    let same_five = Cc::clone(&five);
    ///    let other_five = Cc::new(5);
    ///
    ///    assert!(Cc::ptr_eq(&five, &same_five));
    ///    assert!(!Cc::ptr_eq(&five, &other_five));
    /// }
    /// collect_cycles();
    /// ```
    ///
    /// [`ptr::eq`]: core::ptr::eq
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this._ptr.as_ptr() == other._ptr.as_ptr()
    }
}

impl<T: Trace> Deref for Cc<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        if self.strong_count() > 0 {
            unsafe {
                &self._ptr.as_ref().value
            }
        } else {
            panic!("Invalid access during cycle collection");
        }
    }
}

impl<T: Trace> Drop for Cc<T> {
    /// Drops the `Cc<T>`.
    ///
    /// This will decrement the strong reference count. If the strong reference
    /// count becomes zero and the only other references are `Weak<T>` ones,
    /// `drop`s the inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// {
    ///     let five = Cc::new(5);
    ///
    ///     // stuff
    ///
    ///     drop(five); // explicit drop
    /// }
    /// {
    ///     let five = Cc::new(5);
    ///
    ///     // stuff
    ///
    /// } // implicit drop
    /// ```
    fn drop(&mut self) {
        unsafe {
            if self.strong() > 0 {
                self.dec_strong();
                if self.strong() == 0 {
                    self.release();
                } else {
                    self.possible_root();
                }
            }
        }
    }
}

impl<T: Trace> Clone for Cc<T> {

    /// Makes a clone of the `Cc<T>`.
    ///
    /// When you clone an `Cc<T>`, it will create another pointer to the data and
    /// increase the strong reference counter.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::{Cc, collect_cycles};
    ///
    /// let five = Cc::new(5);
    ///
    /// drop(five.clone());
    /// collect_cycles();
    /// ```
    #[inline]
    fn clone(&self) -> Cc<T> {
        self.inc_strong();
        Cc { _ptr: self._ptr }
    }
}

impl<T: Default + Trace> Default for Cc<T> {
    /// Creates a new `Cc<T>`, with the `Default` value for `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let x: Cc<i32> = Default::default();
    /// ```
    #[inline]
    fn default() -> Cc<T> {
        Cc::new(Default::default())
    }
}

impl<T: PartialEq + Trace> PartialEq for Cc<T> {
    /// Equality for two `Cc<T>`s.
    ///
    /// Two `Cc<T>`s are equal if their inner value are equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let five = Cc::new(5);
    ///
    /// five == Cc::new(5);
    /// ```
    #[inline(always)]
    fn eq(&self, other: &Cc<T>) -> bool { **self == **other }

    /// Inequality for two `Cc<T>`s.
    ///
    /// Two `Cc<T>`s are unequal if their inner value are unequal.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let five = Cc::new(5);
    ///
    /// five != Cc::new(5);
    /// ```
    #[inline(always)]
    fn ne(&self, other: &Cc<T>) -> bool { **self != **other }
}

impl<T: Eq + Trace> Eq for Cc<T> {}

impl<T: PartialOrd + Trace> PartialOrd for Cc<T> {
    /// Partial comparison for two `Cc<T>`s.
    ///
    /// The two are compared by calling `partial_cmp()` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let five = Cc::new(5);
    ///
    /// five.partial_cmp(&Cc::new(5));
    /// ```
    #[inline(always)]
    fn partial_cmp(&self, other: &Cc<T>) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }

    /// Less-than comparison for two `Cc<T>`s.
    ///
    /// The two are compared by calling `<` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let five = Cc::new(5);
    ///
    /// five < Cc::new(5);
    /// ```
    #[inline(always)]
    fn lt(&self, other: &Cc<T>) -> bool { **self < **other }

    /// 'Less-than or equal to' comparison for two `Cc<T>`s.
    ///
    /// The two are compared by calling `<=` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let five = Cc::new(5);
    ///
    /// five <= Cc::new(5);
    /// ```
    #[inline(always)]
    fn le(&self, other: &Cc<T>) -> bool { **self <= **other }

    /// Greater-than comparison for two `Cc<T>`s.
    ///
    /// The two are compared by calling `>` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let five = Cc::new(5);
    ///
    /// five > Cc::new(5);
    /// ```
    #[inline(always)]
    fn gt(&self, other: &Cc<T>) -> bool { **self > **other }

    /// 'Greater-than or equal to' comparison for two `Cc<T>`s.
    ///
    /// The two are compared by calling `>=` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let five = Cc::new(5);
    ///
    /// five >= Cc::new(5);
    /// ```
    #[inline(always)]
    fn ge(&self, other: &Cc<T>) -> bool { **self >= **other }
}

impl<T: Ord + Trace> Ord for Cc<T> {
    /// Comparison for two `Cc<T>`s.
    ///
    /// The two are compared by calling `cmp()` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let five = Cc::new(5);
    ///
    /// five.partial_cmp(&Cc::new(5));
    /// ```
    #[inline]
    fn cmp(&self, other: &Cc<T>) -> Ordering { (**self).cmp(&**other) }
}

// FIXME (#18248) Make `T` `Sized?`
impl<T: Hash + Trace> Hash for Cc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: fmt::Display + Trace> fmt::Display for Cc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug + Trace> fmt::Debug for Cc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: Trace> fmt::Pointer for Cc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self._ptr, f)
    }
}

/// A weak version of `Cc<T>`.
///
/// Weak references do not count when determining if the inner value should be
/// dropped.
///
/// See the [module level documentation](./) for more.
pub struct Weak<T: Trace> {
    // FIXME #12808: strange names to try to avoid interfering with
    // field accesses of the contained type via Deref
    _ptr: NonNull<CcBox<T>>,
}

impl<T: Trace> Weak<T> {

    /// Upgrades a weak reference to a strong reference.
    ///
    /// Upgrades the `Weak<T>` reference to an `Cc<T>`, if possible.
    ///
    /// Returns `None` if there were no strong references and the data was
    /// destroyed.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::{Cc, collect_cycles};
    ///
    /// let five = Cc::new(5);
    ///
    /// let weak_five = five.downgrade();
    ///
    /// let strong_five: Option<Cc<_>> = weak_five.upgrade();
    /// drop((five, weak_five, strong_five));
    /// collect_cycles();
    /// ```
    pub fn upgrade(&self) -> Option<Cc<T>> {
        if self.strong() == 0 {
            None
        } else {
            self.inc_strong();
            Some(Cc { _ptr: self._ptr })
        }
    }
}

impl<T: Trace> Drop for Weak<T> {
    /// Drops the `Weak<T>`.
    ///
    /// This will decrement the weak reference count.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// {
    ///     let five = Cc::new(5);
    ///     let weak_five = five.downgrade();
    ///
    ///     // stuff
    ///
    ///     drop(weak_five); // explicit drop
    /// }
    /// {
    ///     let five = Cc::new(5);
    ///     let weak_five = five.downgrade();
    ///
    ///     // stuff
    ///
    /// } // implicit drop
    /// ```
    fn drop(&mut self) {
        unsafe {
            if self.weak() > 0 {
                self.dec_weak();
                // The weak count starts at 1, and will only go to zero if all
                // the strong pointers have disappeared.
                if self.weak() == 0 {
                    dealloc(self._ptr.cast().as_ptr(), Layout::new::<CcBox<T>>())
                }
            }
        }
    }
}

impl<T: Trace> Clone for Weak<T> {

    /// Makes a clone of the `Weak<T>`.
    ///
    /// This increases the weak reference count.
    ///
    /// # Examples
    ///
    /// ```
    /// use bacon_rajan_cc::Cc;
    ///
    /// let weak_five = Cc::new(5).downgrade();
    ///
    /// weak_five.clone();
    /// ```
    #[inline]
    fn clone(&self) -> Weak<T> {
        self.inc_weak();
        Weak { _ptr: self._ptr }
    }
}

impl<T: fmt::Debug + Trace> fmt::Debug for Weak<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Weak)")
    }
}

impl<T: Trace> Trace for Cc<T> {
    fn trace(&self, tracer: &mut Tracer) {
        unsafe {
            tracer(self._ptr.as_ref());
        }
    }
}

impl<T: Trace> Trace for Weak<T> {
    fn trace(&self, _tracer: &mut Tracer) {
        // Weak references should not be traced.
    }
}

impl<T: Trace> Trace for CcBox<T> {
    fn trace(&self, tracer: &mut Tracer) {
        Trace::trace(&self.value, tracer);
    }
}

#[doc(hidden)]
impl<T: Trace> CcBoxPtr for Cc<T> {
    #[inline(always)]
    fn data(&self) -> &CcBoxData {
        unsafe {
            // Safe to assume this here, as if it weren't true, we'd be breaking
            // the contract anyway.
            // This allows the null check to be elided in the destructor if we
            // manipulated the reference count in the same function.
            &self._ptr.as_ref().data
        }
    }

}

impl<T: Trace> CcBoxPtr for Weak<T> {
    #[inline(always)]
    fn data(&self) -> &CcBoxData {
        unsafe {
            // Safe to assume this here, as if it weren't true, we'd be breaking
            // the contract anyway.
            // This allows the null check to be elided in the destructor if we
            // manipulated the reference count in the same function.

            // We specifically avoid taking a reference to the CcBox because
            // it will cover the containing T and there may already be a mutable
            // reference to it on the stack because we can end up being called
            // from the drop method of strong Cc<T> to the same data.
            // The standard library does the same sort of thing using `WeakInner`
            &(*self._ptr.as_ptr()).data
        }
    }

}

// We also implement CcBoxPtr on CcBox so we can add and operate on type erased CcBox's
// added to the ROOTS table
impl<T: Trace> CcBoxPtr for CcBox<T> {
    #[inline(always)]
    fn data(&self) -> &CcBoxData { &self.data }

}

unsafe fn deallocate(ptr: NonNull<dyn CcBoxPtr>) {
    dealloc(ptr.cast().as_ptr(), Layout::for_value(ptr.as_ref()));
}

pub(crate) unsafe fn drop_value(ptr: NonNull<dyn CcBoxPtr>) {
    ptr::drop_in_place(ptr.as_ptr());
}

#[cfg(test)]
mod tests {
    use super::{Cc, Weak, Trace, Tracer};
    use std::boxed::Box;
    use std::cell::RefCell;
    use std::option::Option;
    use std::option::Option::{Some, None};
    use std::result::Result::{Err, Ok};
    use std::mem::drop;
    use std::clone::Clone;
    use collect::collect_cycles;

    // Tests copied from `Rc<T>`.

    #[test]
    fn test_clone() {
        {
        let x = Cc::new(RefCell::new(5));
        let y = x.clone();
        *x.borrow_mut() = 20;
        assert_eq!(*y.borrow(), 20);
        }
        collect_cycles();
    }

    #[test]
    fn test_simple() {
        let x = Cc::new(5);
        assert_eq!(*x, 5);
    }

    #[test]
    fn test_simple_clone() {
        {
        let x = Cc::new(5);
        let y = x.clone();
        assert_eq!(*x, 5);
        assert_eq!(*y, 5);
        }
        collect_cycles();
    }

    #[test]
    fn test_destructor() {
        let x: Cc<Box<_>> = Cc::new(Box::new(5));
        assert_eq!(**x, 5);
    }

    #[test]
    fn test_live() {
        {
        let x = Cc::new(5);
        let y = x.downgrade();
        assert!(y.upgrade().is_some());
        }
        collect_cycles();
    }

    #[test]
    fn test_dead() {
        let x = Cc::new(5);
        let y = x.downgrade();
        drop(x);
        assert!(y.upgrade().is_none());
    }

    #[test]
    fn weak_self_cyclic() {
        {
        struct Cycle {
            x: RefCell<Option<Weak<Cycle>>>
        }

        impl Trace for Cycle {
            fn trace(&self, _: &mut Tracer) { }
        }

        let a = Cc::new(Cycle { x: RefCell::new(None) });
        let b = a.clone().downgrade();
        *a.x.borrow_mut() = Some(b);
        }
        collect_cycles();
        // hopefully we don't double-free (or leak)...
    }

    #[test]
    fn is_unique() {
        {
        let x = Cc::new(3);
        assert!(x.is_unique());
        let y = x.clone();
        assert!(!x.is_unique());
        drop(y);
        assert!(x.is_unique());
        let w = x.downgrade();
        assert!(!x.is_unique());
        drop(w);
        assert!(x.is_unique());
        }
        collect_cycles();
    }

    #[test]
    fn test_strong_count() {
        {
        let a = Cc::new(0u32);
        assert!(a.strong_count() == 1);
        let w = a.downgrade();
        assert!(a.strong_count() == 1);
        let b = w.upgrade().expect("upgrade of live rc failed");
        assert!(b.strong_count() == 2);
        assert!(b.strong_count() == 2);
        drop(w);
        drop(a);
        assert!(b.strong_count() == 1);
        let c = b.clone();
        assert!(b.strong_count() == 2);
        assert!(c.strong_count() == 2);
        }
        collect_cycles();
    }

    #[test]
    fn test_weak_count() {
        {
        let a = Cc::new(0u32);
        assert!(a.strong_count() == 1);
        assert!(a.weak_count() == 0);
        let w = a.downgrade();
        assert!(a.strong_count() == 1);
        assert!(a.weak_count() == 1);
        drop(w);
        assert!(a.strong_count() == 1);
        assert!(a.weak_count() == 0);
        let c = a.clone();
        assert!(a.strong_count() == 2);
        assert!(a.weak_count() == 0);
        drop(c);
        }
        collect_cycles();
    }

    #[test]
    fn try_unwrap() {
        {
        let x = Cc::new(3);
        assert_eq!(x.try_unwrap(), Ok(3));
        let x = Cc::new(4);
        let _y = x.clone();
        assert_eq!(x.try_unwrap(), Err(Cc::new(4)));
        let x = Cc::new(5);
        let _w = x.downgrade();
        assert_eq!(x.try_unwrap(), Err(Cc::new(5)));
        }
        collect_cycles();
    }

    #[test]
    fn get_mut() {
        {
        let mut x = Cc::new(3);
        *x.get_mut().unwrap() = 4;
        assert_eq!(*x, 4);
        let y = x.clone();
        assert!(x.get_mut().is_none());
        drop(y);
        assert!(x.get_mut().is_some());
        let _w = x.downgrade();
        assert!(x.get_mut().is_none());
        }
        collect_cycles();
    }


    #[test]
    fn test_cowrc_clone_make_unique() {
        {
        let mut cow0 = Cc::new(75);
        let mut cow1 = cow0.clone();
        let mut cow2 = cow1.clone();

        assert!(75 == *cow0.make_unique());
        assert!(75 == *cow1.make_unique());
        assert!(75 == *cow2.make_unique());

        *cow0.make_unique() += 1;
        *cow1.make_unique() += 2;
        *cow2.make_unique() += 3;

        assert!(76 == *cow0);
        assert!(77 == *cow1);
        assert!(78 == *cow2);

        // none should point to the same backing memory
        assert!(*cow0 != *cow1);
        assert!(*cow0 != *cow2);
        assert!(*cow1 != *cow2);
        }
        collect_cycles();
    }

    #[test]
    fn test_cowrc_clone_unique2() {
        {
        let mut cow0 = Cc::new(75);
        let cow1 = cow0.clone();
        let cow2 = cow1.clone();

        assert!(75 == *cow0);
        assert!(75 == *cow1);
        assert!(75 == *cow2);

        *cow0.make_unique() += 1;

        assert!(76 == *cow0);
        assert!(75 == *cow1);
        assert!(75 == *cow2);

        // cow1 and cow2 should share the same contents
        // cow0 should have a unique reference
        assert!(*cow0 != *cow1);
        assert!(*cow0 != *cow2);
        assert!(*cow1 == *cow2);
        }
        collect_cycles();
    }

    #[test]
    fn test_cowrc_clone_weak() {
        {
        let mut cow0 = Cc::new(75);
        let cow1_weak = cow0.downgrade();

        assert!(75 == *cow0);
        assert!(75 == *cow1_weak.upgrade().unwrap());

        *cow0.make_unique() += 1;

        assert!(76 == *cow0);
        assert!(cow1_weak.upgrade().is_none());
        }
        collect_cycles();
    }

    #[test]
    fn test_show() {
        let foo = Cc::new(75);
        assert_eq!(format!("{:?}", foo), "75");
    }

    #[cfg(not(all(target_os = "macos", miri)))]
    #[test]
    fn test_map() {
        let mut map = std::collections::HashMap::new();

        map.insert("Foo".to_string(), 4);

        let x = Cc::new(map);
        assert_eq!(x.get("Foo"), Some(&4));
    }

    #[test]
    fn list_cycle() {
        use std::cell::RefCell;

        struct List(Vec<Cc<RefCell<List>>>);
        impl Trace for List {
            fn trace(&self, tracer: &mut Tracer) {
                self.0.trace(tracer);
            }
        }
        {
            let a = Cc::new(RefCell::new(List(Vec::new())));
            let b = Cc::new(RefCell::new(List(Vec::new())));
            {
                let mut a = a.borrow_mut();
                a.0.push(b.clone());
            }
            {
                let mut b = b.borrow_mut();
                b.0.push(a.clone());
            }
        }
        collect_cycles();
    }

    #[test]
    fn test_retain_weak() {
        let retained_weak_a;
        {
            struct A {
                x: Cc<RefCell<Option<A>>>,
            }
            struct WeakA {
                _x: Weak<RefCell<Option<A>>>
            }
            impl A {
                fn downgrade(this: &Self) -> WeakA {
                    WeakA { _x: Cc::downgrade(&this.x) }
                }
            }
            impl Clone for A {
                fn clone(&self) -> Self {
                    A { x: self.x.clone() }
                }
            }
            impl Trace for A {
                fn trace(&self, tracer: &mut Tracer) {
                    self.x.trace(tracer);
                }
            }
            let a = A { x: Cc::new(RefCell::new(None)) };
            *a.x.borrow_mut() = Some(a.clone());
            retained_weak_a = A::downgrade(&a);
        }
        collect_cycles();
        let _x = retained_weak_a;
    }

    #[test]
    fn test_no_leak_with_double_indirection() {
        use crate::collect::*;
        #[derive(Debug, Clone)]
        struct S {
            ty: Cc<Cc<i32>>,
        }

        // If either of the drops below is missing, we don't get a leak
        let ty = Cc::new(5);
        drop(ty.clone());
        let s = S { ty: Cc::new(ty) };
        drop(s.ty.clone());

        // if collect_cycles() is called before s is dropped, we don't get a leak
        std::mem::drop(s);
        collect_cycles();
    }

    #[test]
    fn test_double_visit_scan_black() {
        let count = std::rc::Rc::new(std::cell::Cell::new(0));
        struct A {
            count: std::rc::Rc<std::cell::Cell<i32>>,
            next_op: Cc<RefCell<Option<A>>>
        }
        impl Clone for A {
            fn clone(&self) -> Self {
                self.count.set(self.count.get() + 1);
                A {
                    count: self.count.clone(),
                    next_op: self.next_op.clone()
                }
            }
        }
        impl Trace for A {
            fn trace(&self, tracer: &mut Tracer) {
                self.next_op.trace(tracer);
            }
        }
        impl A {
            fn new(count: std::rc::Rc<std::cell::Cell<i32>>, next_op: Option<A>) -> A {
                count.set(count.get() + 1);
                A {
                    count,
                    next_op: Cc::new(RefCell::new(next_op))
                }
            }
        }
        impl Drop for A {
            fn drop(&mut self) {
                self.count.set(self.count.get() - 1);
            }
        }
        {
            let q;
            {
                let z = A::new(count.clone(), None);
                let y = A::new(count.clone(), Some(z.clone()));
                let x = A::new(count.clone(), Some(y));
                *z.next_op.borrow_mut() = Some(x.clone());
                q = x;
            }
            collect_cycles();
            *q.next_op.borrow_mut() = None;
        }
        collect_cycles();
        assert_eq!(count.get(), 0);
    }

    #[test]
    fn extra_free() {
        struct Env {
            pub closures: Vec<Cc<RefCell<Clos>>>,
            pub next: Option<Cc<Env>>,
        }
        impl Trace for Env {
            fn trace(&self, tracer: &mut Tracer) {
                self.closures.trace(tracer);
                self.next.trace(tracer);
            }
        }
        struct Clos {
            pub env: Cc<Env>,
        }
        impl Trace for Clos {
            fn trace(&self, tracer: &mut Tracer) {
                self.env.trace(tracer);
            }
        }

        let live_env = {
            let base_env = Cc::new(Env {
                closures: vec![],
                next: None,
            });

            let env_a = Cc::new(Env {
                closures: vec![Cc::new(RefCell::new(Clos {
                    env: base_env.clone(),
                }))],
                next: Some(base_env.clone()),
            });

            let circular_env = Cc::new(Env {
                closures: vec![Cc::new(RefCell::new(Clos {
                    env: base_env.clone(),
                }))],
                next: Some(env_a.clone()),
            });
            circular_env.closures[0].replace(Clos {
                env: circular_env.clone(),
            });

            let live_env = Cc::new(Env {
                closures: vec![],
                next: Some(env_a.clone()),
            });

            drop(base_env); // don't need the stack ref
            drop(env_a); // don't need the stack ref
            collect_cycles();

            drop(circular_env); // cycle root
            collect_cycles(); // <- incorrectly? frees env_a.
                              // mark_gray decrements env_a and does
                              // not reinstate (it's the root of the
                              // black region). collect_white frees
                              // circular_env, which decrements env_a
                              // again - to zero and frees it...

            live_env
        };

        if let Some(a) = &live_env.next {
            assert_eq!(a.closures.len(), 1);
        }
        drop(live_env);
        collect_cycles();
    }

    #[test]
    fn weak_cycle() {
        type Owner = RefCell<Option<Weak<Gadget>>>;
        struct Gadget {
            owner: Cc<Owner>
        }

        impl Trace for Gadget {
            fn trace(&self, tracer: &mut Tracer) {
                tracer(&self.owner);
            }
        }

        let gadget_owner = Cc::new(RefCell::new(None));
        let gadget = Cc::new(Gadget{owner: gadget_owner.clone()});

        *gadget_owner.borrow_mut() = Some(gadget.clone().downgrade());

        drop(gadget_owner);
        drop(gadget);
        collect_cycles();
    }
}
