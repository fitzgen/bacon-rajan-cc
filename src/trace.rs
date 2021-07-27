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
pub type Tracer<'a> = dyn FnMut(&(dyn CcBoxPtr + 'static)) + 'a;

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

/// Mark types as acyclic. Opt-out the cycle collector.
///
/// ## Examples
///
/// ```
/// use gcmodule::trace_acyclic;
///
/// struct X(u32);
/// struct Y(String);
/// struct Z<T>(fn (T));
///
/// trace_acyclic!(X);
/// trace_acyclic!(Y);
/// trace_acyclic!(<T> Z<T>);
/// ```
macro_rules! trace_acyclic {
    ( <$( $g:ident ),*> $( $t: tt )* ) => {
        impl<$( $g: 'static ),*> $crate::Trace for $($t)* {
            fn trace(&self, _tracer: &mut Tracer) { }
        }
    };
    ( $( $t: ty ),* ) => {
        $( trace_acyclic!(<> $t); )*
    };
}

/// Implement `Trace` for simple container types.
///
/// ## Examples
///
/// ```
/// use gcmodule::Trace;
/// use gcmodule::trace_fields;
///
/// struct X<T1, T2> { a: T1, b: T2 };
/// struct Y<T>(Box<T>);
/// struct Z(Box<dyn Trace>);
///
/// trace_fields!(
///     X<T1, T2> { a: T1, b: T2 }
///     Y<T> { 0: T }
///     Z { 0 }
/// );
/// ```
macro_rules! trace_fields {
    ( $( $type:ty { $( $field:tt $(: $tp:ident )? ),* } )* ) => {
        $(
            impl< $( $( $tp: $crate::Trace )? ),* > $crate::Trace for $type {
                fn trace(&self, tracer: &mut $crate::Tracer) {
                    let _ = tracer;
                    $( (&self . $field ).trace(tracer); )*
                }
            }
        )*
    };
}

mod impls {
    pub use super::*;

    mod primitives {
        pub use super::*;

        trace_acyclic!(
            bool,
            char,
            f32,
            f64,
            i16,
            i32,
            i64,
            i8,
            isize,
            u16,
            u32,
            u64,
            u8,
            usize,
            (),
            String,
            str);

        impl<'a, T: Trace> Trace for &'a mut [T] {
            fn trace(&self, tracer: &mut Tracer) {
                for t in &self[..] {
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
            trace_fields!(
                (A, B) { 0: A, 1: B }
                (A, B, C) { 0: A, 1: B, 2: C }
                (A, B, C, D) { 0: A, 1: B, 2: C, 3: D }
                (A, B, C, D, E) { 0: A, 1: B, 2: C, 3: D, 4: E }
            );
        }
    }

    mod boxed {
        pub use super::*;

        impl<T: Trace + ?Sized> Trace for Box<T> {
            fn trace(&self, tracer: &mut Tracer) {
                (**self).trace(tracer);
            }
        }
    }

    mod cell {
        pub use super::*;
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
        pub use super::*;
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
        pub use super::*;
        impl<T: Trace> Trace for Vec<T> {
            fn trace(&self, tracer: &mut Tracer) {
                for t in self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod ffi {
        pub use super::*;
        use std::ffi;

        trace_acyclic!(ffi::CString, ffi::NulError, ffi::OsString);
    }

    mod io {
        pub use super::*;
        use std::io;

        impl<T: io::Write> Trace for io::BufWriter<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
        }

        impl<T: io::Write> Trace for io::LineWriter<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
        }

        trace_acyclic!(<T> io::BufReader<T>);
        trace_acyclic!(<T> io::Cursor<T>);
        trace_acyclic!(<T> io::IntoInnerError<T>);
        trace_acyclic!(<T> io::Lines<T>);
        trace_acyclic!(<T> io::Split<T>);
        trace_acyclic!(<T> io::Take<T>);
        trace_acyclic!(
            io::Empty,
            io::Error,
            io::Repeat,
            io::Sink,
            io::Stderr,
            io::Stdin,
            io::Stdout);
    }

    mod net {
        pub use super::*;
        use std::net;

        trace_acyclic!(
            net::AddrParseError,
            net::Ipv4Addr,
            net::Ipv6Addr,
            net::SocketAddrV4,
            net::SocketAddrV6,
            net::TcpListener,
            net::TcpStream,
            net::UdpSocket
        );
    }

    mod option {
        pub use super::*;

        impl<T: Trace> Trace for Option<T> {
            fn trace(&self, tracer: &mut Tracer) {
                if let Some(ref t) = *self {
                    t.trace(tracer);
                }
            }
        }
    }

    mod path {
        pub use super::*;
        use std::path;

        trace_acyclic!(path::Path, path::PathBuf);
    }

    mod process {
        pub use super::*;
        use std::process;

        trace_acyclic!(
            process::Child,
            process::ChildStderr,
            process::ChildStdin,
            process::ChildStdout,
            process::Command,
            process::ExitStatus,
            process::Output,
            process::Stdio
        );
    }

    mod rc {
        pub use super::*;
        use std::rc;

        impl<T> Trace for rc::Rc<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for rc::Weak<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
        }
    }

    mod result {
        pub use super::*;

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
        pub use super::*;
        use std::sync;

        impl<T> Trace for sync::Arc<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
        }

        impl Trace for sync::Barrier {
            fn trace(&self, _tracer: &mut Tracer) { }
        }

        impl Trace for sync::Condvar {
            fn trace(&self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for sync::Mutex<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
        }

        impl Trace for sync::Once {
            fn trace(&self, _tracer: &mut Tracer) { }
        }

        impl<T> Trace for sync::PoisonError<T> {
            fn trace(&self, _tracer: &mut Tracer) { }
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
        pub use super::*;
        use std::thread;

        trace_acyclic!(<T> thread::JoinHandle<T>);
        trace_acyclic!(<T> thread::LocalKey<T>);
        trace_acyclic!(thread::Thread);
        trace_acyclic!(thread::Builder);
    }
}
