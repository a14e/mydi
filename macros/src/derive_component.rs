use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::ParseStream;
use syn::{parse_macro_input, DeriveInput, Error, Type};
use syn::{Data, DataStruct, Field, Fields, Generics};
use syn::{Result, Token};

pub(crate) fn derive_inject_impl(
    ident: Ident,
    data: Data,
    mut generics: Generics,
) -> syn::Result<TokenStream> {
    let fields: Vec<_> = match &data {
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(named) => named.named.clone(),
            Fields::Unit => Default::default(),
            _ => {
                return Ok(quote_spanned! {
                    ident.span() => compile_error!("you can only derive Inject on structs with named fields or empty structs");
                });
            }
        },
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive Inject on structs");
            });
        }
    }.into_iter()
        .collect();

    let fields_with_types_and_settings: Vec<(TokenStream, Type, DefaultValue)> = fields
        .iter()
        .map(|field| {
            let ident = format_ident!("{}", field.ident.as_ref().unwrap());
            let name = quote!( #ident );
            let typed_name = {
                let field_type = field.ty.clone();
                field_type
            };
            let default_value = read_default_value(field)?;
            Ok((name, typed_name, default_value))
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let defaults: Vec<_> = fields_with_types_and_settings
        .iter()
        .filter(|(_, _, value)| !matches!(value, DefaultValue::None))
        .filter_map(|(filed_name, _, value)| match value {
            DefaultValue::DefaultFunction(func) => Some(quote!(#filed_name: #func())),
            DefaultValue::Default => Some(quote!(#filed_name: Default::default())),
            _ => None,
        })
        .collect();

    let (inject_field, fields_types): (Vec<_>, Vec<_>) = fields_with_types_and_settings
        .into_iter()
        .filter_map(|(field, field_type, default_status)| {
            if let DefaultValue::None = default_status {
                return Some((field, field_type));
            }
            None
        })
        .unzip();

    let defaults = {
        if defaults.is_empty() {
            quote!()
        } else if inject_field.is_empty() {
            quote!(#(#defaults),*)
        } else {
            quote!(,#(#defaults),*)
        }
    };

    // Добавление требования реализации трейта Clone для каждого дженерика
    for param in generics.params.iter_mut() {
        use syn::{GenericParam, TypeParamBound};
        if let GenericParam::Type(type_param) = param {
            type_param
                .bounds
                .push(TypeParamBound::Trait(syn::parse_quote!(Clone)));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote!(

        #[automatically_derived]
        impl #impl_generics mydi::component_meta::ComponentMeta for #ident #ty_generics #where_clause {
            fn inject(injector: &mydi::injector::Injector) -> anyhow::Result<Self>  {
                #(let #inject_field = injector.get()?);*;
                let result = Self {
                    #(#inject_field),*
                    #defaults
                };
                Ok(result)
            }

            fn debug_line() -> Option<String> {
                let line_num = line!();
                let file_name = file!();
                let mut result = String::new();
                result.push_str(file_name);
                result.push_str(":");
                result.push_str(line_num.to_string().as_str());
                Some(result)
            }

            fn dependencies_names() -> Vec<(std::any::TypeId, &'static str)> {
                use std::any::TypeId;
                use std::any::type_name;
                vec! [
                    #( (TypeId::of::<#fields_types>(), type_name::<#fields_types>()) ),*
                ]
            }

        }

    ))
}

#[derive(Debug)]
enum DefaultValue {
    None,
    Default,
    DefaultFunction(proc_macro2::TokenStream),
}

// generated by chat gpt
fn read_default_value(field: &Field) -> Result<DefaultValue> {
    let mut default_value: DefaultValue = DefaultValue::None;

    for attribute in &field.attrs {
        if attribute.path.is_ident("component") {
            if let DefaultValue::None = default_value {
                let component_args = attribute.parse_args_with(|input: ParseStream| {
                    if input.is_empty() {
                        return Err(input.error("Expected an argument after #[component(...)]."));
                    }

                    let _default_keyword: Token![default] = input.parse()?;

                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;

                        if input.peek(syn::Ident) {
                            let default_function: syn::Ident = input.parse()?;
                            Ok(DefaultValue::DefaultFunction(default_function.to_token_stream()))
                        } else {
                            Err(input.error("Expected a user-defined function identifier after #[component(default = ...)]."))
                        }
                    } else {
                        Ok(DefaultValue::Default)
                    }
                });

                match component_args {
                    Ok(value) => default_value = value,
                    Err(err) => return Err(err),
                }
            } else {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "Multiple #[component(...)] annotations found. Ensure the field has only one #[component(...)] annotation.",
                ));
            }
        }
    }

    Ok(default_value)
}
