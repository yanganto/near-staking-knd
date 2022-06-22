//! Consul client implementation

use anyhow::{bail, Context, Result};
use reqwest::Client;
use reqwest::Url;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::str;

/// A client implementing the Consul leader election: https://learn.hashicorp.com/tutorials/consul/application-leader-elections
#[derive(Debug, Clone)]
pub struct ConsulClient {
    client: Client,
    url: Url,
}

/// A consul session according to https://www.consul.io/api-docs/session
#[derive(Debug, Clone, PartialEq)]
pub struct ConsulSession {
    id: String,
}

impl ConsulSession {
    /// Session ID
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
/// Semantically errors returned by consul
pub enum ConsulError {
    /// Session does not (longer) exists
    SessionNotFound,
}
impl std::error::Error for ConsulError {}
impl fmt::Display for ConsulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ConsulClient {
    /// Returns a new Consul client for the given endpoint
    ///
    /// # Arguments
    ///
    /// * `url` - The consul endpoint url
    pub fn new(url: &str) -> Result<ConsulClient> {
        let url = Url::parse(url).with_context(|| "Failed to create consul url")?;
        Ok(ConsulClient {
            client: Client::new(),
            url,
        })
    }

    /// Initializes and returns a new session.
    /// Also see `<https://www.consul.io/api-docs/session#create-session>``
    ///
    /// # Arguments
    ///
    /// * `session_name` - Human readable name of the session
    /// * `ttl` - ttl in seconds
    pub async fn create_session(&self, session_name: &str, ttl: u64) -> Result<ConsulSession> {
        let mut map = HashMap::new();
        map.insert("Name", session_name);
        // How long the session persists until locks are release
        let ttl = format!("{}s", ttl);
        map.insert("TTL", &ttl);

        let url = self
            .url
            .join("/v1/session/create")
            .context("Failed to create session url")?;
        let res = self
            .client
            .put(url)
            .json(&map)
            .send()
            .await
            .context("Failed to create session")?;

        let m = res
            .json::<HashMap<String, String>>()
            .await
            .context("expected Session result to be a hashmap")?;
        let id = m.get("ID").context("no ID field found in session object")?;
        Ok(ConsulSession { id: id.to_string() })
    }

    /// This renews the given consul session. This is used with sessions that have a TTL, and it extends the expiration by the TTL.
    /// Also see `<https://www.consul.io/api-docs/session#renew-session>`
    pub async fn renew_session(&self, session: &ConsulSession) -> Result<()> {
        let url = self
            .url
            .join(&format!("/v1/session/renew/{}", session.id()))
            .context("Failed to create session renew url")?;

        let resp = self
            .client
            .put(url)
            .send()
            .await
            .context("Failed to renew session")?;
        match resp.status() {
            code if code.is_success() => Ok(()),
            reqwest::StatusCode::NOT_FOUND => {
                bail!(ConsulError::SessionNotFound)
            }
            code => {
                let text = resp.text().await.unwrap_or_else(|_| "".to_string());
                bail!(
                    "failed to renew session, consul returned (code: {}): {}",
                    code,
                    text
                )
            }
        }
    }

    /// Delete a given consul session (`<https://www.consul.io/api-docs/session#delete-session>`)
    pub async fn delete_session(&self, session: &ConsulSession) -> Result<()> {
        let url = self
            .url
            .join(&format!("/v1/session/destroy/{}", session.id()))
            .context("Failed to create `destroy session` url")?;

        let res = self
            .client
            .put(url)
            .send()
            .await
            .context("Failed to delete session")?;
        match res.status() {
            code if code.is_success() => Ok(()),
            code => {
                let text = res.text().await.unwrap_or_else(|_| "".to_string());
                bail!(
                    "Failed to delete session, consul returned (code: {}): {}",
                    code,
                    text
                )
            }
        }
    }

    /// This endpoint returns the specified key. If no key exists at the given path,
    /// a 404 is returned instead of a 200 response.
    /// Also see `<https://www.consul.io/api-docs/kv#read-key>`
    pub async fn get(&self, key: &str) -> Result<()> {
        return Ok(());
    }

    /// Acquire a lock for the given key and hold by the given session.
    /// Returns true if the client acquire the session.
    /// Also see `<https://www.consul.io/api-docs/session#delete-session>`
    ///
    /// # Arguments
    ///
    /// * `key` - name of the key to acquire a lock for
    /// * `value` - value to store in the key
    /// * `session` - consul session that tries to acquire the lock
    pub async fn acquire_key<T>(&self, key: &str, value: T, session: &ConsulSession) -> Result<bool>
    where
        T: Serialize,
    {
        let mut url = self
            .url
            .join(&format!("/v1/kv/{}", key))
            .context("Failed to create kv url")?;
        url.set_query(Some(&format!("acquire={}", session.id())));

        let res = self
            .client
            .put(url)
            .json(&value)
            .send()
            .await
            .context("Failed to acquire key")?;
        match res.status() {
            code if code.is_success() => return Ok(res.json::<bool>().await?),
            code => {
                let text = res.text().await.unwrap_or_else(|_| "".to_string());
                bail!(
                    "failed to acquire key, consul returned (code: {}): {}",
                    code,
                    text
                )
            }
        }
    }
}
