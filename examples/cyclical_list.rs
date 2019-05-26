extern crate bacon_rajan_cc;
use bacon_rajan_cc::{collect_cycles, Cc, Trace, Tracer};
use std::cell::RefCell;

struct List(Vec<Cc<RefCell<List>>>);
impl Trace for List {
    fn trace(&mut self, tracer: &mut Tracer) {
        self.0.trace(tracer);
    }
}

fn main() {
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
