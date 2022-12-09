use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::{consul_client::ConsulClient, leader_protocol::consul_leader_key};

/// A consul session according to https://www.consul.io/api-docs/session
// FIXME The fields are here inherited from consul and we probably want to change them a bit...
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Validator {
    /// Fully qualified domain
    #[serde(rename = "Node")]
    pub node: String,
    /// Local name without domain
    #[serde(rename = "Name")]
    pub name: String,
}

pub async fn active_validator(
    account_id: &str,
    consul_url: &str,
    consul_token_file: &Option<PathBuf>,
) -> Result<Option<Validator>> {
    let token = match consul_token_file {
        Some(ref file) => {
            let s = fs::read_to_string(file)
                .with_context(|| format!("cannot read consul token file {}", file.display()))?;
            Some(s.trim_end().to_string())
        }
        None => None,
    };
    let client = ConsulClient::new(consul_url, token.as_deref())
        .context("Failed to create consul client")?;

    let res = client
        .get(&consul_leader_key(account_id))
        .await
        .context("Failed to get leader key from consul")?;
    let value = match res {
        None => {
            info!("No leader found for {}", account_id);
            return Ok(None);
        }
        Some(session) => session,
    };
    let uuid = match value.session {
        None => {
            info!("Last leader session was expired for {}", account_id);
            return Ok(None);
        }
        Some(val) => val,
    };
    let res = client
        .get_session(&uuid)
        .await
        .context("Failed to get leader key from consul")?;

    let session = match res {
        None => {
            info!("Last leader session was expired for {}", account_id);
            return Ok(None);
        }
        Some(session) => session,
    };

    Ok(Some(Validator {
        node: session.node().to_string(),
        name: session.name().to_string(),
    }))
}
