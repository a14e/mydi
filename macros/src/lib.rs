extern crate proc_macro;

mod derive_component;
mod derive_expander;

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::ParseStream;
use syn::{parse_macro_input, DeriveInput, Error, Type};
use syn::{Data, DataStruct, Field, Fields, Generics};
use syn::{Result, Token};

// Inspired by a part of SeaORM: https://github.com/SeaQL/sea-orm/blob/master/sea-orm-macros/src/derives/active_model.rs
// Assistance with macros provided by ChatGPT-4
#[proc_macro_derive(Component, attributes(component))]
pub fn derive_inject(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse_macro_input!(input);

    derive_component::derive_inject_impl(ident, data, generics)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

// Inspired by a part of SeaORM: https://github.com/SeaQL/sea-orm/blob/master/sea-orm-macros/src/derives/active_model.rs
// Assistance with macros provided by ChatGPT-4
#[proc_macro_derive(ComponentExpander, attributes(ignore_expand, nested_expand, force_expand))]
pub fn derive_expansion(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = parse_macro_input!(input);

    derive_expander::derive_expand_impl(ident, data, generics)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
