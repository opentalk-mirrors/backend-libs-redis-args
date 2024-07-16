// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn from_redis_value(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match try_from_redis_value(ast) {
        Ok(k) => k,
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn try_from_redis_value(ast: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let parameter = FromRedisArgsParameters::from_derive_input(&ast)?;

    match parameter.conversion {
        FromRedisValueConversion::Serde => impl_from_redis_value_serde(&ast),
        FromRedisValueConversion::FromStr => impl_from_redis_value_from_str(&ast),
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(from_redis_value))]
struct FromRedisArgsParameters {
    #[darling(flatten)]
    conversion: FromRedisValueConversion,
}

#[derive(Debug, PartialEq, Eq, FromMeta)]
enum FromRedisValueConversion {
    Serde,
    #[darling(rename = "FromStr")]
    FromStr,
}

fn impl_from_redis_value_serde(input: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;

    let expanded = quote! {
        impl #generics ::redis_args::__exports::redis::FromRedisValue for #ident #generics {
            fn from_redis_value(v: &::redis_args::__exports::redis::Value) -> ::redis_args::__exports::redis::RedisResult<Self> {
                match *v {
                    ::redis_args::__exports::redis::Value::Data(ref bytes) => ::redis_args::__exports::serde_json::from_slice(bytes).map_err(|_| {
                        ::redis_args::__exports::redis::RedisError::from(
                            (::redis_args::__exports::redis::ErrorKind::TypeError, "invalid data content")
                        )
                    }),
                    _ => ::redis_args::__exports::redis::RedisResult::Err(
                        ::redis_args::__exports::redis::RedisError::from(
                            (::redis_args::__exports::redis::ErrorKind::TypeError, "invalid data type")
                        )
                    ),
                }
            }
        }
    };
    Ok(TokenStream::from(expanded))
}

fn impl_from_redis_value_from_str(input: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let generics = &input.generics;
    let ident = &input.ident;

    let expanded = quote! {
        impl #generics ::redis_args::__exports::redis::FromRedisValue for #ident #generics {
            fn from_redis_value(v: &::redis_args::__exports::redis::Value) -> ::redis_args::__exports::redis::RedisResult<Self> {
                match *v {
                    ::redis_args::__exports::redis::Value::Data(ref bytes) => {
                        let s = std::str::from_utf8(bytes).map_err(|_| {
                            ::redis_args::__exports::redis::RedisError::from(
                                (::redis_args::__exports::redis::ErrorKind::TypeError, "string is not utf8")
                            )
                        })?;

                        ::std::str::FromStr::from_str(s).map_err(|_| {
                            ::redis_args::__exports::redis::RedisError::from(
                                (::redis_args::__exports::redis::ErrorKind::TypeError, "invalid data type")
                            )
                        })
                    },
                    _ => ::redis_args::__exports::redis::RedisResult::Err(
                        ::redis_args::__exports::redis::RedisError::from(
                            (::redis_args::__exports::redis::ErrorKind::TypeError, "invalid data type")
                        )
                    ),
                }
            }
        }
    };
    Ok(TokenStream::from(expanded))
}
