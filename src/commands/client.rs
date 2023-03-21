use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use hyper::{Body, Client, Method, Request, Response};
use hyperlocal::{UnixClientExt, Uri};
use serde::de::DeserializeOwned;

use super::{active_validator::Validator, ApiResponse, MaintenanceOperation};

async fn parse_response<T: DeserializeOwned>(req: Response<Body>) -> Result<T> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let body =
        String::from_utf8(body_bytes.to_vec()).context("Cannot decode api response as string")?;
    serde_json::from_str(&body)
        .with_context(|| format!("Cannot decodee api response as json: {body}"))
}

/// A client interact with kuutamo
#[derive(Debug)]
pub struct CommandClient {
    socket_path: PathBuf,
}

impl CommandClient {
    /// Returns a new Neard client for the given endpoint
    pub fn new(socket_path: &Path) -> Self {
        Self {
            socket_path: socket_path.to_owned(),
        }
    }

    /// Get active validator
    pub async fn active_validator(&self) -> Result<Option<Validator>> {
        let url = Uri::new(&self.socket_path, "/active_validator").into();
        let res = Client::unix().get(url).await.with_context(|| {
            format!(
                "failed to connect to kneard via {}",
                self.socket_path.display()
            )
        })?;
        let code = res.status();
        if !code.is_success() {
            let resp: ApiResponse = parse_response(res)
                .await
                .context("failed to parse response")?;
            bail!(
                "Request to get active validator failed: {} (status: {})",
                resp.message,
                resp.status
            )
        };
        Ok(Some(
            parse_response(res)
                .await
                .context("cannot parse validator")?,
        ))
    }

    /// Initiatiate or cancel maintenance shutdown or restart
    pub async fn maintenance_operation(
        &self,
        minimum_length: Option<u64>,
        shutdown_at: Option<u64>,
        cancel: bool,
        restart: bool,
    ) -> Result<()> {
        let url = if restart {
            hyperlocal::Uri::new(&self.socket_path, "/maintenance_restart")
        } else {
            hyperlocal::Uri::new(&self.socket_path, "/maintenance_shutdown")
        };

        let body = serde_json::to_string(&MaintenanceOperation {
            minimum_length,
            shutdown_at,
            cancel,
        })?;
        let req = Request::builder()
            .method(Method::POST)
            .uri(url)
            .body(Body::from(body))
            .context("failed to build request")?;

        let res = Client::unix().request(req).await.with_context(|| {
            format!(
                "failed to connect to kneard via {}",
                self.socket_path.display()
            )
        })?;

        let code = res.status();
        let v: ApiResponse = parse_response(res)
            .await
            .context("failed to parse response")?;

        if code.is_success() {
            println!("{}", v.message);
        } else {
            bail!(
                "Request to initiate maintainace shutdown failed: {} (status: {})",
                v.message,
                v.status
            )
        }
        Ok(())
    }

    /// Get maintenance status
    pub async fn maintenance_status(&self) -> Result<String> {
        let url = Uri::new(&self.socket_path, "/maintenance_status").into();
        let res = Client::unix().get(url).await.with_context(|| {
            format!(
                "failed to connect to kneard via {}",
                self.socket_path.display()
            )
        })?;
        let code = res.status();
        let resp: ApiResponse = parse_response(res)
            .await
            .context("failed to parse response")?;
        if !code.is_success() {
            bail!(
                "Request to get active validator failed: {} (status: {})",
                resp.message,
                resp.status
            )
        };
        Ok(resp.message)
    }
}
