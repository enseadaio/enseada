use std::convert::Infallible;
use std::error::Error as StdError;
use std::sync::Arc;

use http::Method;
use slog::Logger;
use tokio::sync::RwLock;
use warp::{Filter, Rejection, Reply};
use warp::body::BodyDeserializeError;
use warp::reject::{InvalidQuery, MethodNotAllowed};
use warp::reply::{json, with_status};

use acl::Enforcer;
use api::error::{Code, ErrorResponse};
use couchdb::Couch;

use crate::config::Configuration;
use crate::config::tls::Tls;
use crate::error::Error;
use crate::ServerResult;
use controller_runtime::ResourceManager;
use api::Resource;

mod auth;
mod resource;
mod telemetry;

pub async fn start(logger: Logger, couch: Couch, cfg: &Configuration, enforcer: Arc<RwLock<Enforcer>>) -> ServerResult {
    let addr = cfg.http().address();

    // Resources
    let users = resource::mount::<users::api::v1alpha1::User>(logger.new(slog::o!()), couch.clone(), enforcer.clone()).await?;
    let policies = resource::mount::<acl::api::v1alpha1::Policy>(logger.new(slog::o!()), couch.clone(), enforcer.clone()).await?;
    let policy_attachments = resource::mount::<acl::api::v1alpha1::PolicyAttachment>(logger.new(slog::o!()), couch.clone(), enforcer.clone()).await?;
    let role_attachments = resource::mount::<acl::api::v1alpha1::RoleAttachment>(logger.new(slog::o!()), couch.clone(), enforcer.clone()).await?;
    let oauth_clients = resource::mount::<oauth::api::v1alpha1::OAuthClient>(logger.new(slog::o!()), couch.clone(), enforcer.clone()).await?;

    let routes = telemetry::routes()
        .or(warp::path("apis")
            .and(auth::mount_can_i(enforcer.clone())
                .or(users)
                .or(policies)
                .or(policy_attachments)
                .or(role_attachments)
                .or(oauth_clients)
            ).recover(handle_rejection))
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_headers(vec![
                    "User-Agent",
                    "Sec-Fetch-Mode",
                    "Referer",
                    "Origin",
                    "Access-Control-Request-Method",
                    "Access-Control-Request-Headers",
                    "Content-Type",
                ])
                .allow_methods(vec![
                    Method::OPTIONS,
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::HEAD,
                    Method::TRACE,
                    Method::CONNECT,
                    Method::PATCH,
                ]),
        )
        .with(warp::log::custom(telemetry::log(logger.clone())))
        .with(warp::log::custom(telemetry::http_metrics));

    let tls = cfg.http().tls();
    let protocol = tls.map_or_else(|| "http", |_| "https");
    slog::info!(logger, "HTTP server listening on {}://{}", protocol, &addr);
    let server = warp::serve(routes);
    if let Some(Tls { cert, key }) = tls {
        server.tls().cert_path(cert).key_path(key).run(addr).await;
    } else {
        server.run(addr).await;
    };

    Ok(())
}

fn with_logger(logger: Logger) -> impl Filter<Extract=(Logger, ), Error=Infallible> + Clone {
    warp::any().map(move || logger.clone())
}

fn with_manager<T: Resource>(manager: ResourceManager<T>) -> impl Filter<Extract=(ResourceManager<T>, ), Error=Infallible> + Clone {
    warp::any().map(move || manager.clone())
}

fn with_enforcer(enforcer: Arc<RwLock<Enforcer>>) -> impl Filter<Extract=(Arc<RwLock<Enforcer>>, ), Error=Infallible> + Clone {
    warp::any().map(move || enforcer.clone())
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;
    let metadata = None;

    eprintln!("{:?}", err);
    if let Some(Error::ApiError { code: err_code, message: err_message }) = err.find::<Error>() {
        code = *err_code;
        message = err_message.clone();
    } else if let Some(err) = err.find::<InvalidQuery>() {
        code = Code::InvalidRequest;
        message = err.to_string();
    } else if let Some(err) = err.find::<BodyDeserializeError>() {
        code = Code::InvalidRequest;
        message = err.source().map_or_else(|| err.to_string(), |source| source.to_string());
    } else if err.is_not_found() || err.find::<MethodNotAllowed>().is_some() {
        code = Code::NotFound;
        message = "not found".to_string();
    } else {
        code = Code::Unknown;
        message = "internal server error".to_string();
    }

    let status = code.to_status();
    let json = json(&ErrorResponse {
        code,
        message,
        metadata,
    });

    Ok(with_status(json, status))
}
