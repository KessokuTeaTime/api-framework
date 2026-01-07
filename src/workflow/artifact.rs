//! Artifacts from GitHub REST API and related functions.

use std::fmt::Display;

use reqwest::{RequestBuilder, header};
use serde::Deserialize;

use crate::{env::GITHUB_TOKEN, workflow::WorkflowRun};

/// Represents artifacts from GitHub REST API.
#[derive(Debug, Deserialize, Clone)]
pub struct Artifacts {
    pub total_count: u8,
    pub artifacts: Vec<Artifact>,
}

/// Represents an artifact from GitHub REST API.
#[derive(Debug, Deserialize, Clone)]
pub struct Artifact {
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub size_in_bytes: u64,
    pub url: String,
    pub archive_download_url: String,
    pub expired: bool,
    pub created_at: Option<String>,
    pub expires_at: Option<String>,
    pub updated_at: Option<String>,
    pub digest: Option<String>,
    pub workflow_run: Option<WorkflowRun>,
}

impl Display for Artifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({} at {})",
            self.name, self.id, self.archive_download_url
        )
    }
}

/// Builds a request for GitHub REST API.
pub fn github_api_request_builder(url: &str) -> RequestBuilder {
    reqwest::Client::new()
        .get(url)
        .header(header::ACCEPT, "application/vnd.github+json")
        .bearer_auth(&*GITHUB_TOKEN)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "KessokuTeaTime-API/1.0")
}
