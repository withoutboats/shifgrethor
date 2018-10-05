use syn::*;
use synstructure::*;
use proc_macro2::*;

pub fn reroot_impl(s: &Structure) -> TokenStream {
    let rerooted = rerooted(s);
    
    let bounds = bounds(s);

    s.gen_impl(quote! {
        extern crate shifgrethor;

        gen unsafe impl<'__root> shifgrethor::Reroot<'__root> for @Self where
            #(#bounds,)*
        {
            type Rerooted = #rerooted;
        }
    })
}

fn bounds<'a>(s: &'a Structure) -> impl Iterator<Item = TokenStream> + 'a {
    s.variants().into_iter().flat_map(|variant| variant.bindings()).map(|b| {
        field_where_clause(b.ast())
    })
}

fn field_where_clause(field: &Field) -> TokenStream {
    let ty = &field.ty;
    let rerooted_ty = fold::fold_type(&mut RootFolder, ty.clone());
    quote! {
        #ty: shifgrethor::Reroot<'__root, Rerooted = #rerooted_ty>
    }
}

fn rerooted(s: &Structure) -> PathSegment {
    fold::fold_path_segment(&mut RootFolder, self_type(s))
}

fn self_type(s: &Structure) -> PathSegment {
    let ident = s.ast().ident.clone();
    let args = s.ast().generics.params.iter().map(|param| {
        match param {
            GenericParam::Lifetime(lt)  => GenericArgument::Lifetime(lt.lifetime.clone()),
            GenericParam::Type(ty)      => {
                let ty = &ty.ident;
                GenericArgument::Type(parse2(quote!(#ty)).unwrap())
            }
            GenericParam::Const(konst)  => {
                let konst = &konst.ident;
                GenericArgument::Const(parse2(quote!(#konst)).unwrap())
            }
        }
    }).collect();

    PathSegment {
        ident,
        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            args,
            colon2_token: None,
            lt_token: token::Lt::new(Span::call_site()),
            gt_token: token::Gt::new(Span::call_site()),
        }),
    }
}

pub struct RootFolder;

impl fold::Fold for RootFolder {
    fn fold_lifetime(&mut self, lifetime: Lifetime) -> Lifetime {
        if lifetime.ident == "root" {
            Lifetime::new("'__root", Span::call_site())
        } else { lifetime }
    }
}
