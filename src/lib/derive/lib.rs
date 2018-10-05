#![recursion_limit="128"]

#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

extern crate proc_macro;

use proc_macro2::TokenStream;

mod accessors;
mod reroot;
mod trace;

use crate::accessors::accessors;
use crate::reroot::reroot_impl;
use crate::trace::trace_impl;

decl_derive!([GC, attributes(gc, finalize)] => gc_derive);

fn gc_derive(s: synstructure::Structure) -> TokenStream {
    let tagged_fields = tagged_fields(&s);
    let accessors = accessors(&s, &tagged_fields[..]);
    let trace_impl = trace_impl(&s);
    let reroot_impl = reroot_impl(&s);
    let gc_impl = gc_impl(&s);
    quote! {
        #accessors
        #trace_impl
        #reroot_impl
        #gc_impl
    }
}

fn gc_impl(s: &synstructure::Structure) -> TokenStream {
    s.gen_impl(quote! {
        extern crate shifgrethor;

        gen impl<'__root> shifgrethor::GC<'__root> for @Self {
        }
    })
}

fn tagged_fields<'a>(s: &'a synstructure::Structure<'a>) -> Vec<&'a synstructure::BindingInfo<'a>> {
    s.variants().iter().flat_map(|v| v.bindings()).filter(|binding| {
        binding.ast().attrs.iter().any(|attr| is_attr(attr, "gc"))
    }).collect()
}

fn is_attr(attr: &syn::Attribute, ident: &str) -> bool {
    attr.path.segments.last().unwrap().value().ident == ident
}
