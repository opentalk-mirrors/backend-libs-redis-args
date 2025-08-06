// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use proc_macro::TokenStream;

mod from_redis_value;
mod to_redis_args;

/// Can be derived by structs or enums in order to allow conversion to redis args.
#[proc_macro_derive(ToRedisArgs, attributes(to_redis_args))]
pub fn derive_to_redis_args(input: TokenStream) -> TokenStream {
    to_redis_args::to_redis_args(input)
}

/// Can be derived by structs or enums in order to allow conversion from redis values.
#[proc_macro_derive(FromRedisValue, attributes(from_redis_value))]
pub fn derive_from_redis_value(input: TokenStream) -> TokenStream {
    from_redis_value::from_redis_value(input)
}
