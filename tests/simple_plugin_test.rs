#![feature(custom_derive, trace_macros, plugin)]
#![plugin(bacon_rajan_cc)]

extern crate bacon_rajan_cc;
use bacon_rajan_cc::*;

trace_macros!(true);

#[derive(Debug)]
#[derive_cc_trace]
struct CycleCollectable {
    a: Cc<u32>,
    b: Cc<String>,
    c: u32,
}

trace_macros!(false);

#[test]
fn test_plugin() {
    let x = CycleCollectable {
        a: Cc::new(5),
        b: Cc::new("hello".into()),
        c: 42
    };

    Trace::trace(&x, &mut |v| {
        println!("traced {:?}", v);
    });

    assert!(false);
}
