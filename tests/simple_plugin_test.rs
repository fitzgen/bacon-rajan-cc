#![feature(custom_derive, trace_macros, plugin)]
#![plugin(bacon_rajan_cc)]

extern crate bacon_rajan_cc;
use bacon_rajan_cc::*;

trace_macros!(true);

#[derive(CcTrace, Debug)]
struct CycleCollectable {
    a: Cc<u32>,
    b: Cc<String>,
}

trace_macros!(false);

#[test]
fn test_plugin() {
    let x = CycleCollectable {
        a: Cc::new(5),
        b: Cc::new("hello".into()),
    };

    CcTrace::trace(&x, &mut |v| {
        println!("traced {:?}", v);
    });

    assert!(false);
}
