// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use darling::{
    ast::{Fields, Style},
    util::Override,
    FromDeriveInput, FromField, FromMeta,
};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

const ATTRIBUTE_NAME: &str = "to_redis_args";

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(to_redis_args))]
struct ToRedisArgsParameters {
    #[darling(flatten)]
    conversion: ToRedisArgsConversion,
    data: darling::ast::Data<darling::util::Ignored, FieldReceiver>,
    ident: syn::Ident,
    generics: syn::Generics,
}

#[derive(Debug, PartialEq, Eq, FromMeta)]
enum ToRedisArgsConversion {
    Serde,
    #[darling(rename = "fmt")]
    Format(Override<syn::LitStr>),
    #[darling(rename = "Display")]
    Display,
}

#[derive(Debug, FromField)]
struct FieldReceiver {
    ident: Option<syn::Ident>,
}

pub(crate) fn to_redis_args(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match try_to_redis_args(ast) {
        Ok(k) => k,
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn try_to_redis_args(ast: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let parameters = ToRedisArgsParameters::from_derive_input(&ast)?;

    match &parameters.conversion {
        ToRedisArgsConversion::Serde => impl_to_redis_args_serde(&ast),
        ToRedisArgsConversion::Format(Override::Explicit(fmt)) => {
            impl_to_redis_args_fmt(&parameters, &fmt.value())
        }
        ToRedisArgsConversion::Format(Override::Inherit) => {
            impl_to_redis_args_fmt(&parameters, "{0}")
        }
        ToRedisArgsConversion::Display => impl_to_redis_args_display(&ast),
    }
}

fn impl_to_redis_args_fmt(
    input: &ToRedisArgsParameters,
    fmt: &str,
) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;
    match &input.data {
        darling::ast::Data::Struct(Fields {
            style: Style::Unit, ..
        }) => Err(syn::Error::new(
            Span::call_site(),
            format!(
                "The #[{ATTRIBUTE_NAME}] attribute can only be attached to structs with fields."
            ),
        )),
        darling::ast::Data::Struct(Fields { fields, .. }) => {
            let format_macro_call = get_named_format_macro_call(fmt, fields);

            let expanded = quote! {
                impl #generics ::redis_args::__exports::redis::ToRedisArgs for #ident #generics {
                    fn write_redis_args<W>(&self, out: &mut W)
                    where
                        W: ?Sized + ::redis_args::__exports::redis::RedisWrite,
                    {
                        out.write_arg(#format_macro_call.as_bytes())
                    }
                }
            };
            Ok(TokenStream::from(expanded))
        }
        darling::ast::Data::Enum(_) => Err(syn::Error::new(
            Span::call_site(),
            format!("#[{ATTRIBUTE_NAME}(fmt)] can only be used with structs"),
        )),
    }
}

fn impl_to_redis_args_serde(input: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;

    let expanded = quote! {
        impl #generics ::redis_args::__exports::redis::ToRedisArgs for #ident #generics {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + ::redis_args::__exports::redis::RedisWrite
            {
                let json_val = ::redis_args::__exports::serde_json::to_vec(self).expect("Failed to serialize");
                out.write_arg(&json_val);
            }
        }
    };
    Ok(TokenStream::from(expanded))
}

fn impl_to_redis_args_display(input: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;

    let expanded = quote! {
        impl #generics ::redis_args::__exports::redis::ToRedisArgs for #ident #generics {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + ::redis_args::__exports::redis::RedisWrite
            {
                out.write_arg_fmt(&self);
            }
        }
    };
    Ok(TokenStream::from(expanded))
}

/// Build the `format` macro call for named struct fields.
///
/// This function will check which fields are used in the format string `fmt` and add these fields to the format macro call.
fn get_named_format_macro_call(fmt: &str, fields: &[FieldReceiver]) -> proc_macro2::TokenStream {
    let field_names: Option<Vec<_>> = fields.iter().map(|field| field.ident.as_ref()).collect();

    let field_args: Vec<proc_macro2::TokenStream> = if let Some(field_names) = field_names {
        // named fields
        field_names
            .iter()
            .filter(|field_ident| fmt.contains(&format!("{{{field_ident}}}")))
            .map(|field_ident| quote! {#field_ident=self.#field_ident})
            .collect()
    } else {
        // tuple structs
        fields
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let i = syn::Index::from(i);
                quote! {self.#i}
            })
            .collect()
    };
    quote! {
        format!(#fmt, #(#field_args),*)
    }
}
