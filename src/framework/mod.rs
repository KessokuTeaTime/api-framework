//! The framework for APIs.

#![cfg(feature = "framework")]

mod state;

pub mod queued_async;

pub use state::*;
