//! Defines the environment variables to use.

#![cfg(feature = "env")]

use crate::static_lazy_lock;

use std::env;

/// Parses an environment variable from [`String`] to something else, wrapping any error in [`anyhow::Error`].
#[macro_export]
macro_rules! parse_env {
    ($key:expr => |$var:ident| $expr:expr) => {
        std::env::var($key)
            .map_err(|e| anyhow::anyhow!(e))
            .and_then(|$var| $expr)
    };
    ($key:expr => |$var:ident| $expr:expr; anyhow) => {
        parse_env!($key => |$var| $expr.map_err(|e| anyhow::anyhow!(e)))
    };
}

pub use parse_env;

static_lazy_lock! {
    pub GITHUB_TOKEN: String = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set in environment");
    "The GitHub token."
}

static_lazy_lock! {
    pub MAX_RETRY: u8 = parse_env!("MAX_RETRY" => |s| s.parse::<u8>(); anyhow).unwrap_or(5);
    "The maximum retry limit for transactions."
}
