//! Module to interact with the neard daemon

use anyhow::{Context, Result};
use near_primitives::views::StatusResponse;
use reqwest::Client;
use reqwest::Url;

/// A client implementing the neard status api
#[derive(Debug)]
pub struct NeardClient {
    client: Client,
    url: Url,
}

impl NeardClient {
    /// Returns a new Neard client for the given endpoint
    ///
    /// # Arguments
    ///
    /// * `url` - The consul endpoint url
    pub fn new(url: &str) -> Result<NeardClient> {
        let url = Url::parse(url).with_context(|| "Failed to create neard url")?;
        Ok(NeardClient {
            client: Client::new(),
            url,
        })
    }

    /// Request neard status
    pub async fn status(&self) -> Result<StatusResponse> {
        let url = self
            .url
            .join("/status")
            .context("Failed to build status url")?;
        let res = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to get status")?;

        Ok(res.json::<StatusResponse>().await?)
    }
}
