use tracing::{error, warn};

use crate::env::MAX_RETRIES;

/// An error that occurs during state operations.
#[allow(clippy::exhaustive_enums)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateError {
    /// The operation should be retried.
    Retry,
    /// The operation has been cancelled.
    Cancelled,
}

/// A specialized [`Result`] type for state operations.
///
/// See: [`StateError`].
pub type StateResult<T> = Result<T, StateError>;

/// An error that occurs when retrying is not allowed.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
