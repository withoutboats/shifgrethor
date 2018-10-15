use synstructure::*;
use proc_macro2::*;

pub fn null_trace_impl(s: &Structure) -> TokenStream {
    if !super::has_attr(s, "null_trace") {
        return quote!()
    }

    let bounds = s.variants().into_iter().flat_map(|v| v.bindings()).map(|b| {
        let ty = &b.ast().ty;
        quote! {
            #ty: shifgrethor::raw::NullTrace
        }
    });

    s.gen_impl(quote! {
        extern crate shifgrethor;

        gen unsafe impl shifgrethor::raw::NullTrace for @Self where
            #(#bounds,)*
        { }
    })
}
