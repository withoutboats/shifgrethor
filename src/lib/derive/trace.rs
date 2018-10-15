use syn::*;
use synstructure::*;
use proc_macro2::*;

enum HasDrop { None, Drop, UnsafeDrop }

pub fn trace_impl(s: &Structure) -> TokenStream {
    let mark_body = s.each(|b| quote!(#b.mark()));
    let manage_body = s.each(|b| quote!(#b.manage()));
    let finalize_body = s.clone().bind_with(|_| BindStyle::RefMut).each(|b| quote!(#b.finalize()));
    let drop = has_drop(s);
    let drop_glue = match &drop {
        HasDrop::None       => quote!(),
        _                   => quote!(shifgrethor::Finalize::finalize(self)),
    };
    let bound = match &drop {
        HasDrop::Drop       => {
            assert!(only_has_root_lifetime(s), "GC'd objects with lifetimes other than 'root must use UnsafeFinalize");
            quote! { for<'__root> Self: shifgrethor::raw::Reroot<'__root> }
        }
        _                   => quote! { },
    };
    s.gen_impl(quote! {
        extern crate shifgrethor;

        gen unsafe impl shifgrethor::raw::Trace for @Self where
            #bound
        {
            unsafe fn mark(&self) {
                match self { #mark_body }
            }
            unsafe fn manage(&self) {
                match self { #manage_body }
            }
            unsafe fn finalize(&mut self) {
                match self { #finalize_body }
                #drop_glue
            }
        }
    })
}

fn has_drop(s: &Structure) -> HasDrop {
    let finalize = super::has_attr(s, "finalize");
    let unsafe_finalize = super::has_attr(s, "unsafe_finalize");
    match (finalize, unsafe_finalize) {
        (true, true)    => panic!("type cannot have both finalize & unsafe_finalize attributes"),
        (true, false)   => HasDrop::Drop,
        (false, true)   => HasDrop::UnsafeDrop,
        (false, false)  => HasDrop::None
    }
}

fn only_has_root_lifetime(s: &Structure) -> bool {
    s.ast().generics.params.iter().filter_map(|param| {
        if let GenericParam::Lifetime(LifetimeDef { lifetime, .. }) = param {
            Some(lifetime)
        } else { None }
    }).all(|l| l.ident == "root")
}
