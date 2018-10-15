use syn::*;
use synstructure::*;
use proc_macro2::*;

pub fn accessors(s: &Structure, gcs: &[&BindingInfo]) -> TokenStream {
    let s_ast: &DeriveInput = s.ast();

    // Collector of bindings with `#[gc]` attributes

    // TODO properly handle enum and tuple structs
    if gcs.len() > 0 && s.variants().len() != 1 {
        panic!("gc_accessor attributes can only be used on structs");
    }


    let accessors: TokenStream = gcs.iter().map(|b| {
        let b_ast: &Field = b.ast();

        let field: &Ident = b_ast.ident.as_ref().unwrap();
        let visibility: &Visibility = &s_ast.vis;
        let method: &Ident = b_ast.ident.as_ref().unwrap();

        let ty: &Type = &b_ast.ty;

        quote! {
            #visibility fn #method<'__root>(self: &'__root shifgrethor::Gc<'__root, Self>) -> <#ty as shifgrethor::raw::Store<'__root>>::Accessor {
                unsafe {
                    shifgrethor::raw::Store::rooted(&self.#field)
                }
            }
        }
    }).collect();

    // inherent impl with all the accessors
    let (impl_generics, ty_generics, where_clauses) = s_ast.generics.split_for_impl();
    let name = &s_ast.ident;
    quote! {
        impl #impl_generics #name #ty_generics #where_clauses {
            #accessors
        }
    }
}
