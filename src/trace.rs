// Copyright 2015 The Rust Project Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::ptr::NonNull;

use crate::cc_box_ptr::CcBoxPtr;

/// A `Tracer` is a callback function that is invoked for each `CcBoxPtr` owned
/// by an instance of something.
pub type Tracer<'a> = dyn FnMut(NonNull<dyn CcBoxPtr>) + 'a;

/// A trait that informs cycle collector how to find memory that is owned by a
/// `Trace` instance and managed by the cycle collector.
pub trait Trace {
    /// Invoke the `Tracer` on each of the `CcBoxPtr`s owned by this `Trace`
    /// instance.
    ///
    /// Failing to invoke the tracer on every owned `CcBoxPtr` can lead to
    /// leaking cycles.
    fn trace(&self, tracer: &mut Tracer);
}

mod impls {
    use super::*;

    mod primitives {
        use super::*;

        macro_rules! impl_prim {
            ($($t:ty,)*) => {
                $(
                    impl Trace for $t {
                        fn trace(&self, _tracer: &mut Tracer) {}
                    }
                )*
            }
        }

        impl_prim! {
            bool,
            char,
            f32,
            f64,
            i8,
            i16,
            i32,
            i64,
            i128,
            isize,
            u8,
            u16,
            u32,
            u64,
            u128,
            usize,
            str,
            (),
        }

        impl<T: Trace + ?Sized> Trace for &'_ T {
            fn trace(&self, tracer: &mut Tracer) {
                (**self).trace(tracer)
            }
        }

        impl<T: Trace + ?Sized> Trace for &'_ mut T {
            fn trace(&self, tracer: &mut Tracer) {
                (**self).trace(tracer)
            }
        }

        impl<T: Trace> Trace for [T] {
            fn trace(&self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }

        // requires 1.51
        // mod arrays {
        //     use super::*;

        //     impl<T: Trace, const N: usize> Trace for [T; N] {
        //         fn trace(&self, tracer: &mut Tracer) {
        //             for t in self {
        //                 t.trace(tracer);
        //             }
        //         }
        //     }
        // }

        mod tuples {
            use super::*;

            // macro for implementing n-ary tuple functions and operations
            // from the core library
            macro_rules! tuple_impls {
                ($(
                    $Tuple:ident {
                        $(($idx:tt) -> $T:ident)+
                    }
                )+) => {
                    $(
                        impl<$($T: Trace),+> Trace for ($($T,)+) where last_type!($($T,)+): ?Sized {
                            fn trace(&self, t: &mut Tracer) {
                                $(
                                    self.$idx.trace(t);
                                )+
                            }
                        }
                    )+
                }
            }

            macro_rules! last_type {
                ($a:ident,) => { $a };
                ($a:ident, $($rest_a:ident,)+) => { last_type!($($rest_a,)+) };
            }

            tuple_impls! {
                Tuple1 {
                    (0) -> A
                }
                Tuple2 {
                    (0) -> A
                    (1) -> B
                }
                Tuple3 {
                    (0) -> A
                    (1) -> B
                    (2) -> C
                }
                Tuple4 {
                    (0) -> A
                    (1) -> B
                    (2) -> C
                    (3) -> D
                }
                Tuple5 {
                    (0) -> A
                    (1) -> B
                    (2) -> C
                    (3) -> D
                    (4) -> E
                }
                Tuple6 {
                    (0) -> A
                    (1) -> B
                    (2) -> C
                    (3) -> D
                    (4) -> E
                    (5) -> F
                }
                Tuple7 {
                    (0) -> A
                    (1) -> B
                    (2) -> C
                    (3) -> D
                    (4) -> E
                    (5) -> F
                    (6) -> G
                }
                Tuple8 {
                    (0) -> A
                    (1) -> B
                    (2) -> C
                    (3) -> D
                    (4) -> E
                    (5) -> F
                    (6) -> G
                    (7) -> H
                }
                Tuple9 {
                    (0) -> A
                    (1) -> B
                    (2) -> C
                    (3) -> D
                    (4) -> E
                    (5) -> F
                    (6) -> G
                    (7) -> H
                    (8) -> I
                }
                Tuple10 {
                    (0) -> A
                    (1) -> B
                    (2) -> C
                    (3) -> D
                    (4) -> E
                    (5) -> F
                    (6) -> G
                    (7) -> H
                    (8) -> I
                    (9) -> J
                }
                Tuple11 {
                    (0) -> A
                    (1) -> B
                    (2) -> C
                    (3) -> D
                    (4) -> E
                    (5) -> F
                    (6) -> G
                    (7) -> H
                    (8) -> I
                    (9) -> J
                    (10) -> K
                }
                Tuple12 {
                    (0) -> A
                    (1) -> B
                    (2) -> C
                    (3) -> D
                    (4) -> E
                    (5) -> F
                    (6) -> G
                    (7) -> H
                    (8) -> I
                    (9) -> J
                    (10) -> K
                    (11) -> L
                }
            }
        }
    }

    mod boxed {
        use super::*;

        impl<T: Trace + ?Sized> Trace for Box<T> {
            fn trace(&self, tracer: &mut Tracer) {
                (**self).trace(tracer);
            }
        }
    }

    mod cell {
        use super::*;
        use std::cell;

        impl<T: Copy + Trace + ?Sized> Trace for cell::Cell<T> {
            fn trace(&self, tracer: &mut Tracer) {
                self.get().trace(tracer);
            }
        }

        impl<T: Trace + ?Sized> Trace for cell::RefCell<T> {
            fn trace(&self, tracer: &mut Tracer) {
                // We'll panic if we can't borrow. I'm not
                // sure if we have a better option.
                self.borrow().trace(tracer);
            }
        }
    }

    mod collections {
        use super::*;
        use std::collections;

        impl<K: Trace, V: Trace> Trace for collections::BTreeMap<K, V> {
            fn trace(&self, tracer: &mut Tracer) {
                for (k, v) in self {
                    k.trace(tracer);
                    v.trace(tracer);
                }
            }
        }

        impl<T: Trace> Trace for collections::BTreeSet<T> {
            fn trace(&self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }

        impl<K: Trace, V: Trace, S> Trace for collections::HashMap<K, V, S> {
            fn trace(&self, tracer: &mut Tracer) {
                for (k, v) in self {
                    k.trace(tracer);
                    v.trace(tracer);
                }
            }
        }

        impl<T: Trace, S> Trace for collections::HashSet<T, S> {
            fn trace(&self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }

        impl<T: Trace> Trace for collections::LinkedList<T> {
            fn trace(&self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }

        impl<T: Trace> Trace for collections::VecDeque<T> {
            fn trace(&self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod vec {
        use super::*;
        impl<T: Trace> Trace for Vec<T> {
            fn trace(&self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod string {
        use super::*;
        impl Trace for String {
            fn trace(&self, _tracer: &mut Tracer) {}
        }
    }

    mod ffi {
        use super::*;
        use std::ffi;

        impl Trace for ffi::CStr {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for ffi::CString {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for ffi::NulError {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for ffi::OsStr {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for ffi::OsString {
            fn trace(&self, _tracer: &mut Tracer) {}
        }
    }

    mod io {
        use super::*;
        use std::io;

        impl<T> Trace for io::BufReader<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T: io::Write> Trace for io::BufWriter<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T> Trace for io::Cursor<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for io::Empty {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for io::Error {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T> Trace for io::IntoInnerError<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T: io::Write> Trace for io::LineWriter<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T> Trace for io::Lines<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for io::Repeat {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for io::Sink {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T> Trace for io::Split<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for io::Stderr {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for io::Stdin {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for io::Stdout {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T> Trace for io::Take<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }
    }

    mod net {
        use super::*;
        use std::net;

        impl Trace for net::AddrParseError {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for net::Ipv4Addr {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for net::Ipv6Addr {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for net::SocketAddrV4 {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for net::SocketAddrV6 {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for net::TcpListener {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for net::TcpStream {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for net::UdpSocket {
            fn trace(&self, _tracer: &mut Tracer) {}
        }
    }

    mod option {
        use super::*;

        impl<T: Trace> Trace for Option<T> {
            fn trace(&self, tracer: &mut Tracer) {
                if let Some(ref t) = *self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod path {
        use super::*;
        use std::path;

        impl Trace for path::Path {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for path::PathBuf {
            fn trace(&self, _tracer: &mut Tracer) {}
        }
    }

    mod process {
        use super::*;
        use std::process;

        impl Trace for process::Child {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for process::ChildStderr {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for process::ChildStdin {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for process::ChildStdout {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for process::Command {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for process::ExitStatus {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for process::Output {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for process::Stdio {
            fn trace(&self, _tracer: &mut Tracer) {}
        }
    }

    mod rc {
        use super::*;
        use std::rc;

        impl<T> Trace for rc::Rc<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T> Trace for rc::Weak<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }
    }

    mod result {
        use super::*;

        impl<T: Trace, U: Trace> Trace for Result<T, U> {
            fn trace(&self, tracer: &mut Tracer) {
                match *self {
                    Ok(ref t) => t.trace(tracer),
                    Err(ref u) => u.trace(tracer),
                }
            }
        }
    }

    mod sync {
        use super::*;
        use std::sync;

        impl<T> Trace for sync::Arc<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for sync::Barrier {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for sync::Condvar {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T> Trace for sync::Mutex<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for sync::Once {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T> Trace for sync::PoisonError<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T: Trace> Trace for sync::RwLock<T> {
            fn trace(&self, tracer: &mut Tracer) {
                if let Ok(v) = self.write() {
                    v.trace(tracer);
                }
            }
        }
    }

    mod thread {
        use super::*;
        use std::thread;

        impl Trace for thread::Builder {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T> Trace for thread::JoinHandle<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl<T> Trace for thread::LocalKey<T> {
            fn trace(&self, _tracer: &mut Tracer) {}
        }

        impl Trace for thread::Thread {
            fn trace(&self, _tracer: &mut Tracer) {}
        }
    }
}
