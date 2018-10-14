#![recursion_limit="128"]

#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

extern crate proc_macro;

mod accessors;
mod null_trace;
mod reroot;
mod trace;

use proc_macro2::*;
use syn::*;
use syn::buffer::TokenBuffer;
use syn::punctuated::Punctuated;

use crate::accessors::accessors;
use crate::null_trace::null_trace_impl;
use crate::reroot::reroot_impl;
use crate::trace::trace_impl;

decl_derive!([GC, attributes(gc)] => gc_derive);

fn gc_derive(s: synstructure::Structure) -> TokenStream {
    let tagged_fields = tagged_fields(&s);
    let accessors = accessors(&s, &tagged_fields[..]);
    let trace_impl = trace_impl(&s);
    let reroot_impl = reroot_impl(&s);
    let null_trace_impl = null_trace_impl(&s);
    let gc_impl = gc_impl(&s);
    quote! {
        #accessors
        #trace_impl
        #reroot_impl
        #null_trace_impl
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

fn has_attr(s: &synstructure::Structure, ident: &str) -> bool {
    if let Some(attr) = s.ast().attrs.iter().find(|attr| is_attr(attr, "gc")) {
        let attr_content = attr.tts.clone().into_iter().next().unwrap();
        if let TokenTree::Group(attr_content) = attr_content { 
            let buffer = TokenBuffer::new2(attr_content.stream());
            let idents = Punctuated::<Ident, token::Comma>::parse_terminated(buffer.begin()).unwrap().0;
            idents.into_iter().any(|i| i == ident)
        } else { false }
    } else { false }
}
