//! Provides a shutdown signal to gracefully shut down the process with configurable actions.
//!
//! See: [`signal`], [`ShutdownAction`]

#![cfg(feature = "shutdown")]

use crate::static_lazy_lock;

use std::{
    fmt::Debug,
    fs,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process,
};
use tokio::{signal, sync::broadcast};
use tracing::{debug, error, info};

static_lazy_lock! {
    /// The broadcast sender to shut down the process.
    pub SHUTDOWN: broadcast::Sender<ShutdownAction> = {
        let (tx, _) = broadcast::channel::<ShutdownAction>(1);
        return tx
    };
}

/// Signals the prcess to shut down.
///
/// # Panics
///
/// Panics when failed to install Ctrl + C signal handler.
pub async fn signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl + C signal handler")
    };

    let mut shutdown = SHUTDOWN.subscribe();

    tokio::select! {
        _ = ctrl_c => {}
        result = shutdown.recv() => if let Ok(action) = result { match action {
        ShutdownAction::Stop => {}
        ShutdownAction::Restart => restart().await,
        ShutdownAction::Update { executable_path } => update(&executable_path).await
        } }
    }
}

/// The action to perform when shutting down a process.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ShutdownAction {
    /// Gracefully stops the process.
    Stop,
    /// Restarts the process in-place.
    Restart,
    /// Updates the process from a new executable file.
    Update {
        /// The path to the new executable file.
        executable_path: PathBuf,
    },
}

async fn restart() {
    info!("restarting…");
    let executable_path = std::env::current_exe()
        .unwrap_or_else(|e| panic!("failed to get current executable path: {e}!"));
    restart_from(executable_path).await
}

async fn restart_from<P>(executable_path: P)
where
    P: AsRef<Path> + Send + Sync + Debug,
{
    let err = process::Command::new(executable_path.as_ref()).exec();
    panic!("unable to restart the process: {err}!");
}

async fn update<P>(executable_path: P)
where
    P: AsRef<Path> + Send + Sync + Debug,
{
    info!("updating from {executable_path:?}…");

    let current_executable_path = std::env::current_exe()
        .unwrap_or_else(|e| panic!("failed to get current executable path: {e}!"));

    match self_replace::self_replace(&executable_path) {
        Ok(_) => {
            debug!(
                "successfully replaced executable file from {executable_path:?}, removing abundant files…"
            );
            drop(fs::remove_file(executable_path));
        }
        Err(err) => {
            error!("failed to replace executable file from {executable_path:?}: {err}")
        }
    }

    restart_from(current_executable_path).await
}
