// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{fmt::Display, str::FromStr};

use redis::{FromRedisValue as _, ToRedisArgs as _};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::Serialize;

#[test]
fn to_redis_args_fmt() {
    #[derive(Debug, ToRedisArgs)]
    #[to_redis_args(fmt = "address:street={street}:housenumber={number}")]
    pub struct Address {
        street: String,
        number: u32,
    }

    let address = Address {
        street: "baker_street".to_string(),
        number: 42,
    };

    assert_eq!(
        address.to_redis_args(),
        "address:street=baker_street:housenumber=42".to_redis_args()
    );
}

#[test]
fn to_redis_args_fmt_tuple() {
    #[derive(Debug, ToRedisArgs)]
    #[to_redis_args(fmt = "address:street={}:housenumber={}")]
    pub struct Address(String, u32);

    let address = Address("baker_street".to_string(), 42);

    assert_eq!(
        address.to_redis_args(),
        "address:street=baker_street:housenumber=42".to_redis_args()
    );
}

#[test]
fn to_redis_args_serde() {
    #[derive(Serialize, ToRedisArgs)]
    #[to_redis_args(serde)]
    pub struct Address {
        street: String,
        number: u32,
    }

    let address = Address {
        street: "baker_street".to_string(),
        number: 42,
    };

    let redis_arg = address.to_redis_args();
    assert_eq!(redis_arg.len(), 1);
    assert_eq!(
        String::from_utf8(redis_arg[0].clone()).unwrap(),
        serde_json::to_string(&address).unwrap()
    );
}

#[test]
fn to_redis_args_display() {
    #[derive(Serialize, ToRedisArgs)]
    #[to_redis_args(Display)]
    pub struct Address {
        street: String,
        number: u32,
    }

    impl Display for Address {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}:{}", self.number, self.street)
        }
    }

    let address = Address {
        street: "baker_street".to_string(),
        number: 42,
    };

    let redis_arg = address.to_redis_args();
    assert_eq!(redis_arg.len(), 1);
    assert_eq!(
        String::from_utf8(redis_arg[0].clone()).unwrap(),
        "42:baker_street"
    );
}

#[test]
fn from_redis_args_serde() {
    #[derive(Serialize, ToRedisArgs)]
    #[to_redis_args(serde)]
    pub struct Address {
        street: String,
        number: u32,
    }

    let address = Address {
        street: "baker_street".to_string(),
        number: 42,
    };

    let redis_arg = address.to_redis_args();
    assert_eq!(redis_arg.len(), 1);
    assert_eq!(
        String::from_utf8(redis_arg[0].clone()).unwrap(),
        serde_json::to_string(&address).unwrap()
    );
}

#[test]
fn from_redis_value_from_str() {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, FromRedisValue)]
    #[from_redis_value(FromStr)]
    pub struct Address {
        street: String,
        number: u32,
    }

    impl FromStr for Address {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let parts = s.split_once(':').ok_or("Failed to split at `:`")?;
            Ok(Address {
                street: parts.1.to_string(),
                number: parts.0.parse().map_err(|_| "Failed to parse number")?,
            })
        }
    }

    let redis_address = redis::Value::BulkString("42:baker_street".as_bytes().to_owned());
    let address = Address::from_redis_value(redis_address).unwrap();
    assert_eq!(
        address,
        Address {
            street: "baker_street".to_string(),
            number: 42,
        }
    )
}
