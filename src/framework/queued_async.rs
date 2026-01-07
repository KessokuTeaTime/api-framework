//! A framework that loops transactions until the max retry times is reached, or a stop signal is received, or a value is returned.

use crate::framework::{StateError, StateResult};

use super::retry_if_possible;

use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    pin::Pin,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicU8, Ordering},
    },
};

use parking_lot::Mutex;
use tracing::{error, info, warn};

#[derive(Debug, Default)]
struct BusinessHolder {
    lock: tokio::sync::Mutex<()>,
    latest_payload_index: AtomicU8,
}

/// Provides extra information for a [`QueuedAsyncFrameworkContext`] business.
#[derive(Debug, Clone)]
pub struct QueuedAsyncFrameworkContext {
    /// The index of the current business. Can be used to determine if a newer business exist.
    pub index: u8,
    /// The name of the current business. Can be used by loggers to distinguish between businesses.
    pub name: String,
    holder: Arc<BusinessHolder>,
}

impl QueuedAsyncFrameworkContext {
    /// Checks if a newer business exist, conforming to a [`StateResult`] with type `T`.
    ///
    /// # Errors
    ///
    /// An error of [`StateError::Cancelled`] is returned if a newer business exist.
    pub fn check<T>(&self, returning: T) -> StateResult<T> {
        let latest_payload_index = &self.holder.latest_payload_index.load(Ordering::SeqCst);
        if self.index < latest_payload_index - 1 {
            warn!(
                "current payload index ({}) is falling behind the latest one ({latest_payload_index}), exiting deployment {}!",
                &self.index, &self.name
            );
            Err(StateError::Cancelled)
        } else {
            Ok(returning)
        }
    }
}

/// A framework that loops transactions until the max retry times is reached, or a stop signal is received, or a value is returned.
///
/// This framework ensures that the latest business is always executed. The ongoing business should check itself constantly in case a newer business arrives. This is achieved through an index that grows with collapsing businesses, and the [`QueuedAsyncFrameworkContext::check`] function along with result propagation.
#[derive(Debug, Default)]
pub struct QueuedAsyncFramework<ID>
where
    ID: Eq + Hash,
{
    businesses: LazyLock<Mutex<HashMap<ID, Arc<BusinessHolder>>>>,
}

impl<ID> QueuedAsyncFramework<ID>
where
    ID: Eq + Hash,
{
    /// Creates a [`QueuedAsyncFramework`].
    pub fn new() -> Self {
        Self {
            businesses: LazyLock::new(|| Mutex::new(HashMap::new())),
        }
    }
}

impl<ID> QueuedAsyncFramework<ID>
where
    ID: Eq + Hash,
{
    /// Runs transactions asynchronously with a distinguishable id. The name of the business will be the display format of the id.
    ///
    /// # Errors
    ///
    /// Returns the final result of the transaction as-is.
    ///
    /// See: [`Self::run_with_name`]
    pub async fn run<F, R>(&self, id: ID, f: F) -> StateResult<R>
    where
        ID: Display,
        F: Fn(QueuedAsyncFrameworkContext) -> Pin<Box<dyn Future<Output = StateResult<R>> + Send>>
            + Send
            + Sync,
    {
        let name = format!("{}", &id);
        self.run_with_name(id, name, f).await
    }

    /// Runs transactions asynchronously with a distinguishable id and a name.
    ///
    /// # Errors
    ///
    /// Returns the final result of the transaction as-is.
    pub async fn run_with_name<F, R>(&self, id: ID, name: String, f: F) -> StateResult<R>
    where
        F: Fn(QueuedAsyncFrameworkContext) -> Pin<Box<dyn Future<Output = StateResult<R>> + Send>>
            + Send
            + Sync,
    {
        let holder = self.businesses.lock().entry(id).or_default().clone();
        let index = holder.latest_payload_index.fetch_add(1, Ordering::SeqCst);
        let context: QueuedAsyncFrameworkContext = QueuedAsyncFrameworkContext {
            index,
            name: name.clone(),
            holder: holder.clone(),
        };

        info!("starting transaction {name}…");
        let mut retry: u8 = 0;
        let _guard = holder.lock.lock().await;

        loop {
            match f(context.clone()).await.and_then(|r| context.check(r)) {
                Ok(result) => {
                    info!("transaction {name} succeed!");
                    holder
                        .latest_payload_index
                        .store(u8::default(), Ordering::SeqCst);
                    return Ok(result);
                }
                Err(StateError::Retry) => match retry_if_possible(&mut retry) {
                    Ok(_) => continue,
                    Err(_) => {
                        error!("transaction {name} failed!");
                        return Err(StateError::Retry);
                    }
                },
                Err(StateError::Cancelled) => {
                    error!("transaction {name} cancelled!");
                    return Err(StateError::Cancelled);
                }
            }
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn example() {
    // defines a framework
    // leverage `LazyLock` to generate a static value
    static FRAMEWORK: LazyLock<QueuedAsyncFramework<i32>> =
        LazyLock::new(QueuedAsyncFramework::new);

    // runs the transaction inside the framework
    let result = FRAMEWORK
        .run(42, |cx| {
            // Pinboxes the transaction and clone the context
            Box::pin(transaction(cx))
        })
        .await;

    assert!(result.is_ok());

    async fn transaction(cx: QueuedAsyncFrameworkContext) -> StateResult<()> {
        // checks if a newer business exist, and stops executing if so
        cx.check(())?;

        // any logic returning a `State` can be unwrapped...
        let greeting = greet().await?;
        // ...while `State::Retry` and `State::Stop` can control the loop directly
        assert!(greeting == "42!");

        // to exit successfully...
        Ok(())
    }

    async fn greet() -> StateResult<String> {
        Ok(String::from("42!"))
    }
}
