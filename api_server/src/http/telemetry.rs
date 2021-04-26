use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use warp::{Filter, Rejection, Reply};
use warp::hyper::Body;
use warp::reply::Response;

use crate::error::Error;
use std::convert::Infallible;
use warp::log::Info;
use slog::Logger;
use std::collections::HashSet;
use opentelemetry::KeyValue;

pub fn routes() -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    let metrics_exporter = crate::telemetry::init_exporter();

    warp::path!("healthz").and_then(health)
        .or(warp::path!("readyz").and_then(health))
        .or(warp::path!("metrics")
            .and(with_metrics_exporter(metrics_exporter))
            .and_then(metrics))
}

async fn health() -> Result<impl Reply, Rejection> {
    Ok("UP".to_string())
}

async fn metrics(exporter: PrometheusExporter) -> Result<impl Reply, Rejection> {
    let encoder = TextEncoder::new();
    let metrics = exporter.registry().gather();
    let mut body = Vec::new();
    encoder.encode(&metrics, &mut body).map_err(Error::from)?;
    Ok(Response::new(Body::from(body)))
}

fn with_metrics_exporter(exporter: PrometheusExporter) -> impl Filter<Extract=(PrometheusExporter, ), Error=Infallible> + Clone {
    warp::any().map(move || exporter.clone())
}

lazy_static! {
    static ref IGNORED_PATHS: HashSet<String> = {
        let mut set = HashSet::new();
        set.insert("/healthz".to_string());
        set.insert("/readyz".to_string());
        set.insert("/metrics".to_string());
        set
    };
}

pub fn log(logger: Logger) -> impl Fn(Info) + Clone {
    move |info: Info| {
        if IGNORED_PATHS.contains(info.path()) {
            return;
        }

        slog::info!(logger, ""; slog::o!(
                "method" => info.method().to_string(),
                "path" => info.path().to_string(),
                "status" => info.status().as_u16(),
            ));
    }
}

pub fn http_metrics(info: Info) {
    if IGNORED_PATHS.contains(info.path()) {
        return;
    }

    let meter = opentelemetry::global::meter("http");
    let counter = meter
        .u64_counter("http_requests_total")
        .with_description("Request counter")
        .init();

    let latency = meter
        .i64_value_recorder("http_requests_duration_seconds")
        .with_description("Request latency")
        .init();

    let labels = vec![
        KeyValue::new("method", info.method().to_string()),
        KeyValue::new("path", info.path().to_string()),
        KeyValue::new("code", info.status().as_str().to_string()),
    ];

    counter.add(1, &labels);
    latency.record(info.elapsed().as_millis() as i64, &labels);
}
