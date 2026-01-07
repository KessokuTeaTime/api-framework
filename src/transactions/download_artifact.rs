use futures::Stream;
use tokio_util::bytes::Bytes;
use tracing::{debug, error, info};

use crate::{
    framework::{StateError, StateResult},
    workflow::artifact::{Artifact, github_api_request_builder},
};

/// Downloads the specified artifact from GitHub.
///
/// # Errors
///
/// Returns an error that instructs retrying or cancelling if downloading the artifact fails.
pub async fn download_artifact(
    artifact: &Artifact,
) -> StateResult<impl Stream<Item = Result<Bytes, reqwest::Error>> + use<>> {
    debug!(
        "requesting download from {}…",
        &artifact.archive_download_url
    );

    match github_api_request_builder(&artifact.archive_download_url)
        .send()
        .await
    {
        Ok(resp) => {
            let stream = resp.bytes_stream();
            info!("requested download from {}", artifact.archive_download_url);
            Ok(stream)
        }
        Err(err) => match err.status() {
            Some(reqwest::StatusCode::GONE) => {
                error!("failed to request download: artifact expired or removed");
                Err(StateError::Cancelled)
            }
            Some(status) => {
                if let Some(reason) = status.canonical_reason() {
                    error!(
                        "failed to request download from {}: {} {reason}",
                        &artifact.archive_download_url,
                        status.as_u16()
                    );
                } else {
                    error!(
                        "failed to request download from {}: {}",
                        &artifact.archive_download_url,
                        status.as_u16()
                    )
                }
                Err(StateError::Retry)
            }
            None => {
                error!(
                    "failed to download artifact at {}",
                    &artifact.archive_download_url
                );
                Err(StateError::Retry)
            }
        },
    }
}
