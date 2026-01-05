use tracing::{error, warn};

/// Unwraps a [`State`].
///
/// This macro accepts a [`State`] value, returns the current scope if the value is either [`State::Retry`] or [`State::Stop`], and exposes the data if the value is [`State::Success`].
///
/// # Examples
///
/// ```rust
/// let value = unwrap!(State::Success(42));
/// assert!(value == 42);
///
/// fn scope() -> State<()> {
///     // This line returns the function with a `State::Stop` immediately
///     let value: i32 = unwrap!(State::Stop);
///
///     // This line will never be executed
///     State::Success(())
/// }
///
/// assert!(scope() == State::Stop);
/// ```
#[macro_export]
macro_rules! unwrap {
    ($expr:expr) => {
        match $expr {
            $crate::framework::State::Success(v) => v,
            $crate::framework::State::Retry => return $crate::framework::State::Retry,
            $crate::framework::State::Stop => return $crate::framework::State::Stop,
        }
    };
}

pub use unwrap;

use crate::env::MAX_RETRIES;

/// A state that controls the flow of data.
#[allow(clippy::exhaustive_enums)]
#[derive(Debug, PartialEq, Eq)]
pub enum State<T> {
    /// The control flow should exit with a value.
    Success(T),
    /// The control flow should retry if possible.
    ///
    /// See: [retry_if_possible]
    Retry,
    /// The control flow should exit immediately.
    Stop,
}

impl<T> State<T> {
    /// Maps the value if [`self`] is [`State::Success`].
    pub fn map<F, R>(self, f: F) -> State<R>
    where
        F: FnOnce(T) -> R,
    {
        match self {
            Self::Success(value) => State::Success(f(value)),
            Self::Retry => State::Retry,
            Self::Stop => State::Stop,
        }
    }

    /// Replaces [`self`] with a same-typed [`State`] if [`self`] is [`State::Success`].
    pub fn replace(self, state: Self) -> Self {
        match self {
            Self::Success(_) => state,
            Self::Retry => Self::Retry,
            Self::Stop => Self::Stop,
        }
    }
}

/// An error that occurs when retrying is not allowed.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum RetryError {
    /// The maximum retry times has been exceeded.
    ExceededMaxRetries,
}

/// Decides whether retrying is allowed based on a provided retry times and the [`MAX_RETRIES`] environment variable.
///
/// # Errors
///
/// [`Err<RetryError>`] is returned if retrying is not allowed, otherwise [`Ok<()>`] is returned.
pub fn retry_if_possible(retry: &mut u8) -> Result<(), RetryError> {
    *retry += 1;
    if *retry > *MAX_RETRIES {
        error!("retried for too many times ({}), stopping!", *MAX_RETRIES);
        Err(RetryError::ExceededMaxRetries)
    } else {
        warn!("retrying… ({retry} / {})", *MAX_RETRIES);
        Ok(())
    }
}
