use std::error::Error as _;
use tracing::{debug, error, info};

use crate::{
    framework::{StateError, StateResult},
    workflow::artifact::{Artifact, Artifacts, github_api_request_builder},
};

/// Fetches artifacts from GitHub using the given parameters.
///
/// # Errors
///
/// Returns an error that instructs retrying or cancelling if fetching the artifacts fails, or the number of fetched artifacts does not match the expected count.
pub async fn fetch_artifacts(
    owner: &str,
    repo: &str,
    run_id: &str,
    count: Option<u8>,
) -> StateResult<Vec<Artifact>> {
    let url =
        format!("https://api.github.com/repos/{owner}/{repo}/actions/runs/{run_id}/artifacts");
    match &count {
        Some(1) => debug!("fetching 1 artifact from {url}…"),
        Some(count) => debug!("fetching {count} artifacts from {url}…"),
        None => debug!("fetching artifacts from {url}…"),
    }

    let response = match github_api_request_builder(&url).send().await {
        Ok(response) => response,
        Err(err) => {
            error!("failed to fetch artifacts from {url}: {err}");
            return match err {
                _ if err.is_connect() || err.is_timeout() => Err(StateError::Retry),
                _ => Err(StateError::Cancelled),
            };
        }
    };

    match response.json::<Artifacts>().await {
        Ok(artifacts) => match artifacts.total_count {
            0 => {
                error!("invalid workflow data: no artifacts at {url}!");
                Err(StateError::Cancelled)
            }
            total_count => match &count {
                Some(count) => match total_count {
                    total_count if total_count < *count => {
                        error!(
                            "invalid workflow data: too little artifacts at {url}! expected {count}, got {total_count}"
                        );
                        Err(StateError::Cancelled)
                    }
                    total_count if total_count > *count => {
                        error!(
                            "invalid workflow data: too many artifacts at {url}! expected {count}, got {total_count}",
                        );
                        Err(StateError::Cancelled)
                    }
                    total_count => {
                        match total_count {
                            1 => info!("fetched 1 artifact from {url}"),
                            count => info!("fetched {count} artifacts from {url}"),
                        }
                        Ok(artifacts.artifacts)
                    }
                },
                None => Ok(artifacts.artifacts),
            },
        },
        Err(err) => {
            error!("failed to parse data from {url}: {err}");

            if let Some(source) = err.source() {
                error!("{source}")
            }

            Err(StateError::Retry)
        }
    }
}
