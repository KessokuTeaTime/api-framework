//! Defines the environment variables to use.

#![cfg(feature = "env")]

use crate::static_lazy_lock;

use std::env;

/// Parses an environment variable from [`String`] to something else, wrapping any error in [`anyhow::Error`].
#[macro_export]
macro_rules! parse_env {
    ($key:expr => |$var:ident| $expr:expr) => {
        $crate::env::__priv_macro_use::env::var($key)
            .map_err(|e| $crate::env::__priv_macro_use::anyhow::anyhow!(e))
            .and_then(|$var| $expr)
    };
    ($key:expr => |$var:ident| $expr:expr; anyhow) => {
        $crate::env::__priv_macro_use::parse_env!($key => |$var| $expr.map_err(|e| $crate::env::__priv_macro_use::anyhow::anyhow!(e)))
    };
}

#[cfg(feature = "env_github_token")]
static_lazy_lock! {
    /// The GitHub token.
    pub GITHUB_TOKEN: String = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set in environment");
}

#[cfg(feature = "env_max_retries")]
static_lazy_lock! {
    /// The maximum retry limit for transactions.
    pub MAX_RETRIES: u8 = parse_env!("MAX_RETRIES" => |s| s.parse::<u8>(); anyhow).unwrap_or(5);
}

#[doc(hidden)]
pub mod __priv_macro_use {
    pub use crate::parse_env;
    pub use anyhow;
    pub use std::env;
}
