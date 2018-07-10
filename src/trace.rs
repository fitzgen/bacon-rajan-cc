// Copyright 2015 The Rust Project Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT
// or http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use cc_box_ptr::CcBoxPtr;

/// A `Tracer` is a callback function that is invoked for each `CcBoxPtr` owned
/// by an instance of something.
pub type Tracer = FnMut(&mut CcBoxPtr);

/// A trait that informs cycle collector how to find memory that is owned by a
/// `Trace` instance and managed by the cycle collector.
pub trait Trace {
    /// Invoke the `Tracer` on each of the `CcBoxPtr`s owned by this `Trace`
    /// instance.
    ///
    /// Failing to invoke the tracer on every owned `CcBoxPtr` can lead to
    /// leaking cycles.
    fn trace(&mut self, tracer: &mut Tracer);
}

mod impls {
    pub use super::*;

    mod primitives {
        pub use super::*;

        impl Trace for bool {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for char {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for f32 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for f64 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for i16 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for i32 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for i64 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for i8 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for isize {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for str {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for u16 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for u32 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for u64 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for u8 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for usize {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<'a, T: Trace> Trace for &'a mut [T] {
            fn trace(&mut self, tracer: &mut Tracer) {
                for t in &mut self[..] {
                    t.trace(tracer);
                }
            }
        }

        mod arrays {
            pub use super::*;

            // impl<T: Trace> Trace for [T; 0] {
            // }
            // impl<T: Trace> Trace for [T; 1] {
            // }
            // impl<T: Trace> Trace for [T; 2] {
            // }
            // impl<T: Trace> Trace for [T; 3] {
            // }
            // impl<T: Trace> Trace for [T; 4] {
            // }
            // impl<T: Trace> Trace for [T; 5] {
            // }
            // impl<T: Trace> Trace for [T; 6] {
            // }
            // impl<T: Trace> Trace for [T; 7] {
            // }
            // impl<T: Trace> Trace for [T; 8] {
            // }
            // impl<T: Trace> Trace for [T; 9] {
            // }
            // impl<T: Trace> Trace for [T; 10] {
            // }
            // impl<T: Trace> Trace for [T; 11] {
            // }
            // impl<T: Trace> Trace for [T; 12] {
            // }
            // impl<T: Trace> Trace for [T; 13] {
            // }
            // impl<T: Trace> Trace for [T; 14] {
            // }
            // impl<T: Trace> Trace for [T; 15] {
            // }
            // impl<T: Trace> Trace for [T; 16] {
            // }
            // impl<T: Trace> Trace for [T; 17] {
            // }
            // impl<T: Trace> Trace for [T; 18] {
            // }
            // impl<T: Trace> Trace for [T; 19] {
            // }
            // impl<T: Trace> Trace for [T; 20] {
            // }
            // impl<T: Trace> Trace for [T; 21] {
            // }
            // impl<T: Trace> Trace for [T; 22] {
            // }
            // impl<T: Trace> Trace for [T; 23] {
            // }
            // impl<T: Trace> Trace for [T; 24] {
            // }
            // impl<T: Trace> Trace for [T; 25] {
            // }
            // impl<T: Trace> Trace for [T; 26] {
            // }
            // impl<T: Trace> Trace for [T; 27] {
            // }
            // impl<T: Trace> Trace for [T; 28] {
            // }
            // impl<T: Trace> Trace for [T; 29] {
            // }
            // impl<T: Trace> Trace for [T; 30] {
            // }
            // impl<T: Trace> Trace for [T; 31] {
            // }
            // impl<T: Trace> Trace for [T; 32] {
            // }
        }

        mod tuples {
            // impl Trace for tuple {
            // }
        }
    }

    mod boxed {
        pub use super::*;

        impl<T: Trace> Trace for Box<T> {
            fn trace(&mut self, tracer: &mut Tracer) {
                (**self).trace(tracer);
            }
        }
    }

    mod cell {
        pub use super::*;
        use std::cell;

        impl<T: Copy + Trace> Trace for cell::Cell<T> {
            fn trace(&mut self, tracer: &mut Tracer) {
                self.get().trace(tracer);
            }
        }

        impl<T: Trace> Trace for cell::RefCell<T> {
            fn trace(&mut self, tracer: &mut Tracer) {
                // If the RefCell is currently borrowed we
                // assume there's an outstanding reference to this
                // cycle so it's ok if we don't trace through it.
                // If the borrow gets leaked somehow then we're going
                // to leak the cycle.
                if let Ok(mut x) = self.try_borrow_mut() {
                    x.trace(tracer);
                }
            }
        }
    }

    mod collections {
        pub use super::*;
        use std::collections;
        use std::hash;

        impl<K, V: Trace> Trace for collections::BTreeMap<K, V> {
            fn trace(&mut self, tracer: &mut Tracer) {
                for (_, v) in self {
                    v.trace(tracer);
                }
            }
        }

        impl<K: Eq + hash::Hash + Trace, V: Trace> Trace for collections::HashMap<K, V> {
            fn trace(&mut self, tracer: &mut Tracer) {
                for (_, v) in self {
                    v.trace(tracer);
                }
            }
        }

        impl<T: Trace> Trace for collections::LinkedList<T> {
            fn trace(&mut self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }

        impl<T: Trace> Trace for collections::VecDeque<T> {
            fn trace(&mut self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod vec {
        pub use super::*;
        impl<T: Trace> Trace for Vec<T> {
            fn trace(&mut self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod ffi {
        pub use super::*;
        use std::ffi;

        impl Trace for ffi::CStr {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for ffi::CString {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for ffi::NulError {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for ffi::OsStr {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for ffi::OsString {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }
    }

    mod io {
        pub use super::*;
        use std::io;

        impl<T> Trace for io::BufReader<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T: io::Write> Trace for io::BufWriter<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for io::Cursor<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for io::Empty {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for io::Error {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for io::IntoInnerError<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T: io::Write> Trace for io::LineWriter<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for io::Lines<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for io::Repeat {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for io::Sink {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for io::Split<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for io::Stderr {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for io::Stdin {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for io::Stdout {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for io::Take<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }
    }

    mod net {
        pub use super::*;
        use std::net;

        impl Trace for net::AddrParseError {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for net::Ipv4Addr {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for net::Ipv6Addr {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for net::SocketAddrV4 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for net::SocketAddrV6 {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for net::TcpListener {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for net::TcpStream {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for net::UdpSocket {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }
    }

    mod option {
        pub use super::*;

        impl<T: Trace> Trace for Option<T> {
            fn trace(&mut self, tracer: &mut Tracer) {
                if let Some(ref mut t) = *self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod path {
        pub use super::*;
        use std::path;

        impl Trace for path::Path {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for path::PathBuf {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }
    }

    mod process {
        pub use super::*;
        use std::process;

        impl Trace for process::Child {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for process::ChildStderr {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for process::ChildStdin {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for process::ChildStdout {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for process::Command {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for process::ExitStatus {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for process::Output {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for process::Stdio {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }
    }

    mod rc {
        pub use super::*;
        use std::rc;

        impl<T> Trace for rc::Rc<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for rc::Weak<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }
    }

    mod result {
        pub use super::*;

        impl<T: Trace, U: Trace> Trace for Result<T, U> {
            fn trace(&mut self, tracer: &mut Tracer) {
                match *self {
                    Ok(ref mut t) => t.trace(tracer),
                    Err(ref mut u) => u.trace(tracer),
                }
            }
        }
    }

    mod sync {
        pub use super::*;
        use std::sync;

        impl<T> Trace for sync::Arc<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for sync::Barrier {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for sync::Condvar {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for sync::Mutex<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for sync::Once {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for sync::PoisonError<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T: Trace> Trace for sync::RwLock<T> {
            fn trace(&mut self, tracer: &mut Tracer) {
                if let Ok(mut v) = self.write() {
                    v.trace(tracer);
                }
            }
        }
    }

    mod thread {
        pub use super::*;
        use std::thread;

        impl Trace for thread::Builder {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for thread::JoinHandle<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for thread::LocalKey<T> {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }

        impl Trace for thread::Thread {
            fn trace(&mut self, _tracer: &mut Tracer) { }
        }
    }
}
