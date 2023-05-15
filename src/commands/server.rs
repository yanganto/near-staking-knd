use std::{
    fmt::Display,
    fs,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
};

use anyhow::{Context, Result};
use hyper::{
    header::{HeaderValue, CONTENT_TYPE},
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use hyperlocal::UnixServerExt;
use log::warn;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use tokio::sync::mpsc::{self, Sender};

use crate::{ipc, near_client::NeardClient, settings::Settings, supervisor::SHUTDOWN_WITH_NEARD};

use super::{active_validator::active_validator, ScheduleRestartOperation};

fn server_error<T: Display>(msg: T) -> Response<Body>
where
    T: Display,
{
    warn!("server error '{}'", msg);
    let res = serde_json::to_vec(&json!({"status": 500, "message": format!("{msg}")}));
    let mut resp: Response<Body> = Response::default();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // The builder interface requires unwrap, which I don't like
    *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    let body = match res {
        Err(e) => {
            warn!("Failed to serialize json: {e}");
            Body::from(r#"{"status": 500, "message": "Cannot serialize json"}"#)
        }
        Ok(body) => Body::from(body),
    };
    *resp.body_mut() = body;
    resp
}

fn gateway_timeout<T: Display>(msg: T) -> Response<Body>
where
    T: Display,
{
    warn!("gateway timeout '{}'", msg);
    let res = serde_json::to_vec(&json!({"status": 504, "message": format!("{msg}")}));
    let mut resp: Response<Body> = Response::default();
    resp.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let body = match res {
        Err(e) => {
            warn!("Failed to serialize json: {e}");
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Body::from(r#"{"status": 500, "message": "Cannot serialize json"}"#)
        }
        Ok(body) => {
            *resp.status_mut() = StatusCode::GATEWAY_TIMEOUT;
            Body::from(body)
        }
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
    near_client: NeardClient,
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
    pub fn new(settings: &Settings, supervisor_request_chan: Sender<ipc::Request>) -> Result<Self> {
        Ok(CommandServer {
            account_id: settings.account_id.to_string(),
            consul_url: settings.consul_url.to_string(),
            consul_token_path: settings.consul_token_file.to_owned(),
            control_socket: settings.control_socket.to_owned(),
            supervisor_request_chan,
            near_client: NeardClient::new(&format!(
                "http://localhost:{}",
                settings.near_rpc_addr.port()
            ))?,
        })
    }

    async fn handle_requests(&self, req: Request<Body>) -> hyper::Result<Response<Body>> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/health") => Ok(Response::new(Body::from(
                r#"{"status": 200, "message": "OK"}"#,
            ))),
            (&Method::GET, "/active_validator") => self.handle_active_validator().await,
            (&Method::POST, "/schedule_restart") => self.handle_schedule_restart(req).await,
            (&Method::GET, "/maintenance_status") => self.handle_maintenance_status().await,
            (&Method::GET, "/rpc_status") => self.handle_rpc_status().await,
            _ => Ok(not_found()),
        }
    }

    async fn handle_schedule_restart(&self, req: Request<Body>) -> hyper::Result<Response<Body>> {
        let (tx, mut rx) = mpsc::channel(1);
        let args: ScheduleRestartOperation = ok_or_500!(json_request(req).await);
        let req = ipc::Request::ScheduleRestartOperation(
            args.minimum_length,
            args.schedule_at,
            args.cancel,
            tx,
        );

        if let Err(e) = self.supervisor_request_chan.send(req).await {
            return Ok(server_error(format!(
                "channel to supervisor was already closed before sending: {e}"
            )));
        }

        match rx.recv().await {
            Some(r) => match (r.shutdown_at_blockheight, args.cancel) {
                (Ok(Some(height)), false) => Ok(Response::new(Body::from(format!(
                    "{{\"status\": 200, \"message\": \"will shutdown at block height: {height}\"}}",
                )))),
                (Ok(None), true) => Ok(Response::new(Body::from(
                    r#"{"status": 200, "message": "shutdown cancelled"}"#,
                ))),
                (Ok(None), false) => Ok(Response::new(Body::from(
                    r#"{"status": 200, "message": "is shutting down at current block"}"#,
                ))),
                (Err(e), false) => Ok(server_error(format!("fail to schedule restart: {e:}"))),
                (Err(e), true) => Ok(server_error(format!("fail to cancel restart: {e:}"))),
                (Ok(Some(_)), true) => Ok(server_error(
                    "unexpedted response for cancel restart".to_string(),
                )),
            },
            None => Ok(server_error("channel to supervisor was closed")),
        }
    }

    async fn handle_maintenance_status(&self) -> hyper::Result<Response<Body>> {
        let (metrics, final_block) =
            tokio::join!(self.near_client.metrics(), self.near_client.final_block());

        let resp = match (
            metrics.ok().and_then(|m| {
                m.get("near_block_expected_shutdown")
                    .and_then(|s| s.parse::<u64>().ok())
            }),
            final_block,
            SHUTDOWN_WITH_NEARD.load(Ordering::Acquire),
        ) {
            (Some(expect), Ok(current), true) if expect > 0 => Response::new(Body::from(format!(
                "{{\"status\": 200, \"message\": \"maintenance shutdown in {} blocks, current: {current:}, shutdown at: {expect}\"}}",
                expect - current
            ))),
            (Some(expect), Ok(current), false) if expect > 0 => Response::new(Body::from(format!(
                "{{\"status\": 200, \"message\": \"maintenance restart in {} blocks,  current: {current:}, restart at: {expect}\"}}",
                expect - current
            ))),
            (Some(expect), Err(_), true) if expect > 0 => Response::new(Body::from(format!("{{\"status\": 200, \"message\": \"maintenance shutdown will be at {expect}\"}}"))),
            (Some(expect), Err(_), false) if expect > 0 => Response::new(Body::from(format!("{{\"status\": 200, \"message\": \"maintenance restart will be at {expect}\"}}"))),
            (_, Ok(current), true) => Response::new(Body::from(format!(
                "{{\"status\": 200, \"message\": \"maintenance shutdown set, current: {current:}\"}}",
            ))),
            (_, _, false) => Response::new(Body::from("{\"status\": 200, \"message\": \"no maintenance setting now\"}")),
            (_, Err(_), _) => gateway_timeout("fail to fetch current block from neard"),
        };

        Ok(resp)
    }

    async fn handle_rpc_status(&self) -> hyper::Result<Response<Body>> {
        let resp = match self.near_client.status().await {
            Ok(_) => Response::new(Body::from(
                "{\"status\": 200, \"message\": \"rpc service ready\"}",
            )),
            Err(_) => gateway_timeout("fail to fetch status from rpc service"),
        };
        Ok(resp)
    }

    async fn handle_active_validator(&self) -> hyper::Result<Response<Body>> {
        let validator =
            active_validator(&self.account_id, &self.consul_url, &self.consul_token_path).await;
        Ok(json_response(ok_or_500!(validator)))
    }
}

/// Starts an control socket server
pub async fn spawn_control_server(settings: &Settings, tx: Sender<ipc::Request>) -> Result<()> {
    let server = Arc::new(CommandServer::new(settings, tx)?);
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
