use syn::{parse_macro_input, DeriveInput, Error, Type};
use syn::{Field, Data, Fields, DataStruct, Generics};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote_spanned, quote};
use syn::{Result, Token};
use syn::parse::{ParseStream};
use quote::ToTokens;


pub(crate) fn derive_expand_impl(ident: Ident,
                                 data: Data,
                                 generics: Generics) -> syn::Result<TokenStream> {
    let fields: Vec<_> = match &data {
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(named) => named.named.clone(),
            Fields::Unit => Default::default(),
            _ => {
                return Ok(quote_spanned! {
                    ident.span() => compile_error!("You can only derive Inject on structs with named fields or empty structs");
                });
            }
        },
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("You can only derive Inject on structs");
            });
        }
    }.into_iter()
        .collect();


    if fields.iter().any(|f| ignore_expand(f) && nested_expand(f)) {
        return Ok(quote_spanned! {
                    ident.span() => compile_error!("Can't ignore and expand field simultaneously");
                });
    }

    if fields.iter().any(|f| ignore_expand(f) && force_expand(f)) {
        return Ok(quote_spanned! {
                    ident.span() => compile_error!("Can't ignore and force expanding for field simultaneously");
                });
    }

    let fields_to_extract: Vec<TokenStream> = fields.iter()
        .filter(|x| !ignore_expand(x))
        .map(|field| {
            let ident = format_ident!("{}", field.ident.as_ref().unwrap());
            let name = quote!( #ident );
            name
        })
        .collect::<Vec<_>>();

    let fields_unnested: Vec<TokenStream> = fields.iter()
        .filter(|x| !ignore_expand(x) && !nested_expand(x) && !force_expand(x))
        .map(|field| {
            let ident = format_ident!("{}", field.ident.as_ref().unwrap());
            let name = quote!( #ident );
            name
        })
        .collect::<Vec<_>>();

    let forced_extraction: Vec<TokenStream> = fields.iter()
        .filter(|x| !ignore_expand(x) && force_expand(x))
        .map(|field| {
            let ident = format_ident!("{}", field.ident.as_ref().unwrap());
            let name = quote!( #ident );
            name
        })
        .collect::<Vec<_>>();

    let fields_nested: Vec<TokenStream> = fields.iter()
        .filter(|x| !ignore_expand(x) && nested_expand(x))
        .map(|field| {
            let ident = format_ident!("{}", field.ident.as_ref().unwrap());
            let name = quote!( #ident );
            name
        })
        .collect::<Vec<_>>();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote!(

        #[automatically_derived]
        impl #impl_generics mydi::expander::ComponentExpander for #ident #ty_generics #where_clause {
            fn expand<INJECTION_BINDER_TYPE: Clone + 'static>(self, injector: mydi::injection_binder::InjectionBinder<INJECTION_BINDER_TYPE>) -> mydi::injection_binder::InjectionBinder<INJECTION_BINDER_TYPE>  {
                let Self {#( #fields_to_extract),* , .. } = self;

                injector #(
                    .instance(#fields_unnested)
                )*
                #(
                    .instance(#forced_extraction.clone())
                )*
                #(
                    .expand(#fields_nested)
                )*
            }

        }

    ))
}

fn ignore_expand(field: &Field) -> bool {
    for attribute in &field.attrs {
        if attribute.path.is_ident("ignore_expand") ||
            attribute.path.is_ident("mydi::ignore_expand") {
            return true;
        }
    }

    false
}


fn nested_expand(field: &Field) -> bool {
    for attribute in &field.attrs {
        if attribute.path.is_ident("nested_expand") ||
            attribute.path.is_ident("mydi::nested_expand") {
            return true;
        }
    }

    false
}


fn force_expand(field: &Field) -> bool {
    for attribute in &field.attrs {
        if attribute.path.is_ident("force_expand") ||
            attribute.path.is_ident("mydi::force_expand") {
            return true;
        }
    }

    false
}