//! Prometheus http exporter

use std::time::Instant;

use anyhow::{Context, Result};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use log::warn;
use prometheus::{self, register_gauge, Encoder, Gauge, TextEncoder};

use crate::proc::get_neard_pid;

lazy_static! {
    static ref START: Instant = Instant::now();
    static ref UPTIME: Gauge = register_gauge!(
        "kuutamod_uptime",
        "Time in milliseconds how long daemon is running"
    )
    .unwrap();
}

async fn response_examples(req: Request<Body>) -> hyper::Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => Ok(Response::new(Body::from("OK"))),
        (&Method::GET, "/neard-pid") => match get_neard_pid() {
            Ok(Some(pid)) => Ok(Response::new(Body::from(pid.as_raw().to_string()))),
            Ok(None) => Ok(Response::new(Body::from(""))),
            Err(e) => {
                warn!("Failed to get pid: {}", e);
                Ok(server_error())
            }
        },
        (&Method::GET, "/metrics") => {
            UPTIME.set(START.elapsed().as_millis() as f64);

            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            let encoder = TextEncoder::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            Ok(Response::new(Body::from(buffer)))
        }
        _ => Ok(not_found()),
    }
}

static SERVER_ERROR: &[u8] = b"Server error";

fn server_error() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(SERVER_ERROR.into())
        .unwrap()
}

static NOTFOUND: &[u8] = b"Not Found";
/// HTTP status code 404
fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}

/// Starts an prometheus exporter backend
pub async fn spawn_prometheus_exporter(exporter_address: &str) -> Result<()> {
    lazy_static::initialize(&START);
    let addr = exporter_address
        .parse()
        .context("Failed to parse exporter")?;
    let make_service =
        make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(response_examples)) });
    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{addr}");

    server.await.context("Failed to start server")
}
