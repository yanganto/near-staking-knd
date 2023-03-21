//! Consul client implementation

use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use reqwest::Url;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::str;
use std::sync::Mutex;

/// A client implementing the Consul leader election: https://learn.hashicorp.com/tutorials/consul/application-leader-elections
#[derive(Debug)]
pub struct ConsulClient {
    client: Client,
    url: Url,
    headers: Mutex<HeaderMap>,
}

/// Behavior to take when a session is invalidated
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionBehavior {
    /// Causes any locks that are held to be released
    Release,
    /// Causes any locks that are held to be deleted
    Delete,
}

/// A consul session according to https://www.consul.io/api-docs/session
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConsulSession {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Node")]
    node: String,
    #[serde(rename = "LockDelay")]
    lock_delay: u64,
    #[serde(rename = "Behavior")]
    behavior: SessionBehavior,
    #[serde(rename = "TTL")]
    ttl: String,
    #[serde(rename = "NodeChecks")]
    node_checks: Option<Vec<String>>,
    #[serde(rename = "ServiceChecks")]
    service_checks: Option<Vec<String>>,
    #[serde(rename = "CreateIndex")]
    create_index: u64,
    #[serde(rename = "ModifyIndex")]
    modify_index: u64,
}

impl ConsulSession {
    /// Session ID
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Session Name
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Node Name
    pub fn node(&self) -> &str {
        &self.node
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
        write!(f, "{self:?}")
    }
}

/// Return type of a consul key
/// This is what a normal session looks like
/// [
///   {
///     "LockIndex": 1,
///     "Key": "kneard-leader",
///     "Flags": 0,
///     "Value": "eyJIb3N0bmFtZSI6ImV2ZSIsIk5vZGVQdWJsaWNLZXkiOiJlZDI1NTE5OjdxZ1JhTlI5Z2ZXcVpTejVIc1NHOFhMRlR2ZFcxTHRaRmZqMnJ5RVlFRk5XIiwiTm9kZUlkIjoia3V1dGFtb2QwIn0=",
///     "Session": "bb8c1bc7-cc43-d20a-db4b-b1ecdc7f5572",
///     "CreateIndex": 8,
///     "ModifyIndex": 8
///   }
/// ]
/// This is an expired session:
/// [{
///    "LockIndex": 1,
///    "Key": "kneard-leader",
///    "Flags": 0,
///    "Value": "eyJIb3N0bmFtZSI6ImV2ZSIsIk5vZGVQdWJsaWNLZXkiOiJlZDI1NTE5OjdxZ1JhTlI5Z2ZXcVpTejVIc1NHOFhMRlR2ZFcxTHRaRmZqMnJ5RVlFRk5XIiwiTm9kZUlkIjoia3V1dGFtb2QwIn0=",
///    "CreateIndex": 8,
///    "ModifyIndex": 5103
/// }]
#[derive(Deserialize, Debug, Clone)]
pub struct ConsulValue {
    /// Number of times this key has successfully been acquired in a lock.
    #[serde(rename = "LockIndex")]
    pub lock_index: u64,
    /// Full path of the entry.
    #[serde(rename = "Key")]
    pub key: String,
    /// An opaque unsigned integer that can be attached to each entry. Clients
    /// can choose to use this however makes sense for their application.
    #[serde(rename = "Flags")]
    pub flags: u64,
    /// Base64-encoded blob of data.
    #[serde(rename = "Value")]
    pub value: String,
    /// Session that owns the lock.
    #[serde(rename = "Session")]
    pub session: Option<String>,
    /// Internal index value that represents when the entry was created.
    #[serde(rename = "CreateIndex")]
    pub create_index: u64,
    /// The last index that modified this key.
    #[serde(rename = "ModifyIndex")]
    pub modify_index: u64,
}

impl ConsulClient {
    /// Returns a new Consul client for the given endpoint
    ///
    /// # Arguments
    ///
    /// * `url` - The consul endpoint url
    pub fn new(url: &str, token: Option<&str>) -> Result<ConsulClient> {
        let url = Url::parse(url).with_context(|| "Failed to create consul url")?;
        let client = ConsulClient {
            client: Client::new(),
            url,
            headers: Mutex::new(HeaderMap::new()),
        };
        client.set_token(token)?;
        Ok(client)
    }

    /// Set consul auth token.
    pub fn set_token(&self, token: Option<&str>) -> Result<()> {
        let mut headers = match self.headers.lock() {
            Ok(h) => h,
            Err(e) => {
                bail!("Cannot get header lock: {}", e);
            }
        };
        if let Some(token) = token {
            headers.insert(
                "X-Consul-Token",
                HeaderValue::from_str(token).context("invalid consul token")?,
            );
        } else {
            headers.remove("X-Consul-Token");
        }
        Ok(())
    }

    fn headers(&self) -> Result<HeaderMap> {
        match self.headers.lock() {
            Ok(h) => Ok(h.clone()),
            Err(e) => {
                bail!("Cannot get header lock: {}", e);
            }
        }
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
        let ttl = format!("{ttl}s");
        map.insert("TTL", &ttl);
        // Delete old locks
        map.insert("Behavior", "delete");
        // Delete locks without delay if ttl or session is expired. kneard
        // will stop validating before the ttl expires.
        map.insert("LockDelay", "0s");

        let url = self
            .url
            .join("/v1/session/create")
            .context("Failed to create session url")?;
        let res = self
            .client
            .put(url)
            .headers(self.headers()?)
            .json(&map)
            .send()
            .await
            .context("Failed to create session")?;
        let code = res.status();
        if !code.is_success() {
            let text = res.text().await.unwrap_or_else(|_| "".to_string());
            bail!(
                "failed to create session, consul returned (code: {}): {}",
                code,
                text
            )
        }

        let m = res
            .json::<HashMap<String, String>>()
            .await
            .context("expected Session result to be a hashmap")?;
        let id = m.get("ID").context("no ID field found in session object")?;

        Ok(ConsulSession {
            id: id.to_string(),
            name: session_name.to_string(),
            node: "".to_string(),
            lock_delay: 0,
            behavior: SessionBehavior::Delete,
            ttl,
            node_checks: None,
            service_checks: None,
            create_index: 0,
            modify_index: 0,
        })
    }

    /// Returns the requested session information.
    /// Returns None if no such session is exists or it has been expired
    ///
    /// # Arguments
    ///
    /// * `uuid` - UUID of the session to read
    pub async fn get_session(&self, uuid: &str) -> Result<Option<ConsulSession>> {
        let url = self
            .url
            .join(&format!("/v1/session/info/{uuid}"))
            .context("Failed to get session url")?;
        let res = self
            .client
            .get(url)
            .headers(self.headers()?)
            .send()
            .await
            .context("Failed to get session")?;

        let code = res.status();
        if !code.is_success() {
            let text = res.text().await.unwrap_or_else(|_| "".to_string());
            bail!(
                "failed to get session, consul returned (code: {}): {}",
                code,
                text
            )
        }

        let session = res
            .json::<Vec<ConsulSession>>()
            .await
            .context("Not a valid Session object returned by consul")?;
        if session.is_empty() {
            Ok(None)
        } else {
            Ok(Some(session[0].clone()))
        }
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
            .headers(self.headers()?)
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
            .headers(self.headers()?)
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

    /// This endpoint returns the specified key.
    /// Returns Ok(None) if no key exists at the given path.
    /// Also see `<https://www.consul.io/api-docs/kv#read-key>`
    pub async fn get(&self, key: &str) -> Result<Option<ConsulValue>> {
        let url = self
            .url
            .join(&format!("/v1/kv/{key}"))
            .context("Failed to create kv url")?;
        let res = self
            .client
            .get(url)
            .headers(self.headers()?)
            .send()
            .await
            .context("Failed to get key")?;
        match res.status() {
            code if code.is_success() => {
                let val = res
                    .json::<Vec<ConsulValue>>()
                    .await
                    .context("Failed to decode response")?;
                Ok(Some(val[0].clone()))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            code => {
                let text = res.text().await.unwrap_or_else(|_| "".to_string());
                bail!(
                    "Failed to get key, consul returned (code: {}): {}",
                    code,
                    text
                );
            }
        }
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
            .join(&format!("/v1/kv/{key}"))
            .context("Failed to create kv url")?;
        url.set_query(Some(&format!("acquire={}", session.id())));

        let res = self
            .client
            .put(url)
            .headers(self.headers()?)
            .json(&value)
            .send()
            .await
            .context("Failed to acquire key")?;
        match res.status() {
            code if code.is_success() => Ok(res.json::<bool>().await?),
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
