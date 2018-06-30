#[cfg(not(test))] extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate syn;
extern crate synstructure;

#[cfg(not(test))]
#[proc_macro_derive(Trace, attributes(ignore_trace))]
pub fn expand_token_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_string(&input.to_string()).parse().unwrap()
}

fn expand_string(input: &str) -> String {
    let mut type_ = syn::parse_macro_input(input).unwrap();

    let style = synstructure::BindStyle::RefMut.into();
    let match_body = synstructure::each_field(&mut type_, &style, |binding| {
        if let syn::Ty::Array(..) = binding.field.ty {
            Some(quote! {
                for item in #binding.iter() {
                    ::bacon_rajan_cc::Trace::trace(item, tracer);
                }
            })
        } else {
            Some(quote! {
                ::bacon_rajan_cc::Trace::trace(#binding, tracer);
            })
        }
    });

    let name = &type_.ident;
    let (impl_generics, ty_generics, where_clause) = type_.generics.split_for_impl();
    let mut where_clause = where_clause.clone();
    for param in &type_.generics.ty_params {
        where_clause.predicates.push(syn::WherePredicate::BoundPredicate(syn::WhereBoundPredicate {
            bound_lifetimes: Vec::new(),
            bounded_ty: syn::Ty::Path(None, param.ident.clone().into()),
            bounds: vec![syn::TyParamBound::Trait(
                syn::PolyTraitRef {
                    bound_lifetimes: Vec::new(),
                    trait_ref: syn::parse_path("::bacon_rajan_cc::Trace").unwrap(),
                },
                syn::TraitBoundModifier::None
            )],
        }))
    }

    let tokens = quote! {
        impl #impl_generics ::bacon_rajan_cc::Trace for #name #ty_generics #where_clause {
            #[inline]
            #[allow(unused_variables, unused_mut, unreachable_code)]
            fn trace(&mut self, tracer: &mut Tracer) {
                match *self {
                    #match_body
                }
            }
        }
    };

    tokens.to_string()
}

#[test]
fn test_struct() {
    let mut source = "struct Foo<T> { bar: Bar, baz: T }";
    let mut expanded = expand_string(source);
    let mut no_space = expanded.replace(" ", "");
    macro_rules! match_count {
        ($e: expr, $count: expr) => {
            assert_eq!(no_space.matches(&$e.replace(" ", "")).count(), $count,
                       "counting occurences of {:?} in {:?} (whitespace-insensitive)",
                       $e, expanded)
        }
    }
    match_count!("struct", 0);
    match_count!("impl<T> ::bacon_rajan_cc::Trace for Foo<T> where T: ::bacon_rajan_cc::Trace {", 1);
    match_count!("::bacon_rajan_cc::Trace::trace(", 2);

    source = "struct Bar([Baz; 3]);";
    expanded = expand_string(source);
    no_space = expanded.replace(" ", "");
    match_count!("for item in", 1);
}
