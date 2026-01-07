use crate::{framework::StateResult, transactions::fetch_artifacts, workflow::artifact::Artifact};

/// Fetches the only artifact from GitHub using the given parameters.
///
/// # Errors
///
/// Returns an error that instructs retrying or cancelling if fetching the artifact fails, or the number of fetched artifacts is not exactly one.
pub async fn fetch_artifact(owner: &str, repo: &str, run_id: &str) -> StateResult<Artifact> {
    fetch_artifacts(owner, repo, run_id, Some(1))
        .await
        .map(|artifacts| artifacts[0].clone())
}
