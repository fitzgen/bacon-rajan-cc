// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! TODO FITZGEN
//!
//! ```ignore
//! #![feature(plugin)]
//! #![plugin(bacon_rajan_cc)]
//!
//! fn main() {
//!     TODO FITZGEN
//! }
//! ```


use rustc::plugin::Registry;
use syntax::ast::{Expr, Item, MetaItem, Mutability};
use syntax::ext::base::{Annotatable, Decorator, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ext::deriving::generic::{combine_substructure, EnumMatching, FieldInfo, MethodDef,
                                     Struct, Substructure, TraitDef, ty};
use syntax::codemap::Span;
use syntax::parse::token::intern;
use syntax::ptr::P;

pub fn expand_derive_cc_trace(cx: &mut ExtCtxt,
                              sp: Span,
                              meta_item: &MetaItem,
                              item: &Item,
                              push: &mut FnMut(P<Item>))
{
    println!("FITZGEN HELLO");
    let ann = Annotatable::Item(P(item.clone()));

    let cc_trace_trait_def = TraitDef {
        span: sp,
        attributes: Vec::new(),
        path: ty::Path::new(vec!("bacon_rajan_cc", "Trace")),
        additional_bounds: Vec::new(),
        generics: ty::LifetimeBounds::empty(),
        methods: vec!(
            MethodDef {
                name: "trace",
                generics: ty::LifetimeBounds::empty(),
                explicit_self: ty::borrowed_explicit_self(),
                args: vec!(ty::Ptr(Box::new(ty::Literal(ty::Path::new(vec!("bacon_rajan_cc", "Tracer")))),
                                   ty::PtrTy::Borrowed(None, Mutability::MutMutable))),
                ret_ty: ty::nil_ty(),
                attributes: vec![quote_attr!(cx, #[inline(always)])],
                is_unsafe: false,
                combine_substructure: combine_substructure(Box::new(|a, b, c| {
                    cc_trace_substructure(a, b, c)
                }))
            }
        ),
        associated_types: Vec::new(),
    };

    let mut push2 = |a: Annotatable| {
        push(a.expect_item());
    };
    cc_trace_trait_def.expand(cx, meta_item, &ann, &mut push2);
}

fn cc_trace_substructure(cx: &mut ExtCtxt, trait_span: Span, substr: &Substructure) -> P<Expr> {
    let state_expr = match (substr.nonself_args.len(), substr.nonself_args.get(0)) {
        (1, Some(o_f)) => o_f,
        _ => cx.span_bug(trait_span, "incorrect number of arguments in `derive_cc_trace`")
    };

    let call_cc_trace = |span, thing_expr| {
        let cc_trace_path = {
            let strs = vec!(
                cx.ident_of("bacon_rajan_cc"),
                cx.ident_of("Trace"),
                cx.ident_of("trace"),
            );

            cx.expr_path(cx.path_global(span, strs))
        };
        let ref_thing = cx.expr_addr_of(span, thing_expr);
        let expr = cx.expr_call(span, cc_trace_path, vec!(ref_thing, state_expr.clone()));
        cx.stmt_expr(expr)
    };
    let mut stmts = Vec::new();

    let fields = match *substr.fields {
        Struct(ref fs) | EnumMatching(_, _, ref fs) => fs,
        _ => cx.span_bug(trait_span, "impossible substructure in `derice_cc_trace`")
    };

    for &FieldInfo { ref self_, span, .. } in fields.iter() {
        stmts.push(call_cc_trace(span, self_.clone()));
    }

    cx.expr_block(cx.block(trait_span, stmts, None))
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("derive_cc_trace"), Decorator(Box::new(expand_derive_cc_trace)));
}
