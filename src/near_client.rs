//! Module to interact with the neard daemon

use anyhow::{Context, Result};
use near_primitives::{account::id::AccountId, types::BlockHeight, views::StatusResponse};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// The result for maintenance windows rpc
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MaintenanceWindowRPCResult(pub Vec<(BlockHeight, BlockHeight)>);

/// The rpc result for maintenance windows rpc
#[derive(Deserialize)]
pub struct MaintenanceWindowJsonRpcStatusResponse {
    /// RPC version
    pub jsonrpc: String,
    /// id, may take care in future
    pub id: String,
    /// the result we care
    pub result: MaintenanceWindowRPCResult,
}

/// The rpc result for maintenance windows rpc
#[derive(Deserialize)]
pub struct BlockDetailJsonRpcStatusResponse {
    /// RPC version
    pub jsonrpc: String,
    /// id, may take care in future
    pub id: String,
    /// the result we care
    pub result: BlockHeight,
}

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

    fn rpc_request(method: &str, params: HashMap<String, serde_json::Value>) -> serde_json::Value {
        json!({
             "jsonrpc": "2.0",
             "method": method,
             "id": "dontcare",
             "params": params,
        })
    }

    /// Request final block details
    pub async fn final_block(&self) -> Result<BlockHeight> {
        let mut params = HashMap::<String, serde_json::Value>::new();
        params.insert("finality".into(), "final".into());
        let res = self
            .client
            .post(self.url.clone())
            .json(&Self::rpc_request("block", params))
            .send()
            .await
            .context("Failed to get block details")?;

        let r: BlockDetailJsonRpcStatusResponse = res.json().await?;
        Ok(r.result)
    }

    /// Request maintenance windows
    pub async fn maintenance_windows(
        &self,
        account_id: &AccountId,
    ) -> Result<MaintenanceWindowRPCResult> {
        let mut params = HashMap::<String, serde_json::Value>::new();
        params.insert("account_id".into(), account_id.as_str().into());
        let res = self
            .client
            .post(self.url.clone())
            .json(&Self::rpc_request(
                "EXPERIMENTAL_maintenance_windows",
                params,
            ))
            .send()
            .await
            .context("Failed to get maintenance windows")?;

        let r: MaintenanceWindowJsonRpcStatusResponse = res.json().await?;
        Ok(r.result)
    }

    /// Request metrics
    pub async fn metrics(&self) -> Result<HashMap<String, String>> {
        let url = self
            .url
            .join("/metrics")
            .context("Failed to build metrics url")?;
        let res = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to get metric")?;
        let body = res.text().await?;
        let mut metrics = HashMap::new();
        for line in body.split('\n') {
            if line.starts_with('#') {
                continue;
            } else {
                let mut sp = line.split(' ');
                if let Some(key) = sp.next() {
                    metrics.insert(key.into(), sp.last().unwrap_or("").into());
                }
            }
        }
        Ok(metrics)
    }
}
