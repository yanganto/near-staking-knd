use std::{fmt::Display, fs, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use hyperlocal::UnixServerExt;
use log::warn;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use tokio::sync::mpsc::{self, Sender};

use crate::{ipc, settings::Settings};

use super::{active_validator::active_validator, MaintenanceShutdown};

fn server_error<T: Display>(msg: T) -> Response<Body>
where
    T: Display,
{
    warn!("server error '{}' on {}:{}", msg, file!(), line!());
    let res = serde_json::to_vec(&json!({"status": 500, "message": format!("{}", msg)}));
    // FIXME: Set header: Content-Type: application/json
    let mut resp: Response<Body> = Response::default();

    // The builder interface requires unwrap, which I don't like
    *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    let body = match res {
        Err(e) => {
            warn!("Failed to serialize json: {}", e);
            Body::from(r#"{"status": 500, "message": "Cannot serialize json"}"#)
        }
        Ok(body) => Body::from(body),
    };
    *resp.body_mut() = body;
    resp
}

macro_rules! ok_or_500 {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(err) => {
                return Ok(server_error(err));
            }
        }
    };
}

static NOTFOUND: &[u8] = br#"{"status": 404, "message": "Not Found"}"#;

/// HTTP status code 404
fn not_found() -> Response<Body> {
    // FIXME: Set header: Content-Type: application/json
    let mut resp: Response<Body> = Response::default();
    *resp.status_mut() = StatusCode::NOT_FOUND;
    *resp.body_mut() = Body::from(NOTFOUND);
    resp
}

/// A unix-socket based http server to provide remote control
struct CommandServer {
    account_id: String,
    consul_url: String,
    consul_token_path: Option<PathBuf>,
    control_socket: PathBuf,
    supervisor_request_chan: Sender<ipc::Request>,
}

fn json_response<T: Serialize>(obj: T) -> Response<Body> {
    match serde_json::to_vec(&obj) {
        Err(e) => server_error(e),
        // FIXME: Set header: Content-Type: application/json
        Ok(v) => Response::new(Body::from(v)),
    }
}

async fn json_request<T: DeserializeOwned>(mut req: Request<Body>) -> Result<T> {
    let body = hyper::body::to_bytes(req.body_mut())
        .await
        .context("error receiving body")?;
    let output: T = serde_json::from_slice(&body).context("error converting from json")?;
    Ok(output)
}

impl CommandServer {
    /// Creates a new instance
    pub fn new(settings: &Settings, supervisor_request_chan: Sender<ipc::Request>) -> Self {
        CommandServer {
            account_id: settings.account_id.to_string(),
            consul_url: settings.consul_url.to_string(),
            consul_token_path: settings.consul_token_file.to_owned(),
            control_socket: settings.control_socket.to_owned(),
            supervisor_request_chan,
        }
    }

    async fn handle_requests(&self, req: Request<Body>) -> hyper::Result<Response<Body>> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/health") => Ok(Response::new(Body::from(
                r#"{"status": 200, "message": "OK"}"#,
            ))),
            (&Method::GET, "/active_validator") => self.handle_active_validator().await,
            (&Method::POST, "/maintainance_shutdown") => {
                self.handle_maintenance_shutdown(req).await
            }
            _ => Ok(not_found()),
        }
    }

    async fn handle_maintenance_shutdown(
        &self,
        req: Request<Body>,
    ) -> hyper::Result<Response<Body>> {
        let (tx, mut rx) = mpsc::channel(1);
        let args: MaintenanceShutdown = ok_or_500!(json_request(req).await);
        let req = ipc::Request::MaintenanceShutdown(args.minimum_length, args.shutdown_at, tx);

        if let Err(e) = self.supervisor_request_chan.send(req).await {
            return Ok(server_error(format!(
                "channel to supervisor was already closed before sending: {}",
                e
            )));
        }

        match rx.recv().await {
            Some(_) => Ok(Response::new(Body::from(
                r#"{"status": 200, "message": "OK"}"#,
            ))),
            None => Ok(server_error("channel to supervisor was closed")),
        }
    }
    async fn handle_active_validator(&self) -> hyper::Result<Response<Body>> {
        let validator =
            active_validator(&self.account_id, &self.consul_url, &self.consul_token_path).await;
        Ok(json_response(ok_or_500!(validator)))
    }
}

/// Starts an control socket server
pub async fn spawn_control_server(settings: &Settings, tx: Sender<ipc::Request>) -> Result<()> {
    let server = Arc::new(CommandServer::new(settings, tx));
    let server = &server;

    if server.control_socket.exists() {
        fs::remove_file(&server.control_socket)?;
    }

    let make_service = make_service_fn(move |_client| {
        let server = server.clone();

        async move {
            // This is the request handler.
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let s = Arc::clone(&server);
                async move { s.handle_requests(req).await }
            }))
        }
    });
    let s = Server::bind_unix(&server.control_socket)
        .with_context(|| {
            format!(
                "failed to bind unix socket '{}'",
                server.control_socket.display()
            )
        })?
        .serve(make_service);

    println!("Listening on unix://{}", server.control_socket.display());

    s.await.context("Failed to start server")?;
    Ok(())
}
