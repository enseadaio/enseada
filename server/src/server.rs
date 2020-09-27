use io::BufReader;
use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom};
use std::sync::Arc;

use actix_cors::Cors;
use actix_session::CookieSession;
use actix_web::cookie::SameSite;
use actix_web::middleware::errhandlers::ErrorHandlers;
use actix_web::middleware::normalize::TrailingSlash;
use actix_web::middleware::{Compress, DefaultHeaders, Logger, NormalizePath};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use http::StatusCode;
use rustls::internal::pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig};
use tokio::sync::RwLock;
use url::Url;

use ::rbac::{Enforcer, Watcher};
use events::EventBus;
use oauth;

use crate::config::Configuration;
use crate::couchdb::{self, name as dbname};
use crate::http::error;
use crate::{dashboard, observability, oci, routes, user};

pub async fn run(cfg: &'static Configuration) -> io::Result<()> {
    let address = format!("0.0.0.0:{}", cfg.port());
    let public_host: &Url = cfg.public_host();
    let secret_key = cfg.secret_key();
    let tls = cfg.tls();

    let couch = couchdb::from_config(cfg);
    let rbac_db = couch.database(dbname::RBAC, true);

    let mut enforcer = Enforcer::new(rbac_db.clone());
    enforcer.load_rules().await.expect("Enforcer::load_rules()");
    let enforcer = Arc::new(RwLock::new(enforcer));
    let watcher = Watcher::new(rbac_db.clone(), enforcer.clone());
    watcher.start().expect("Watcher::start()");

    let event_bus = Arc::new(std::sync::RwLock::new(EventBus::new()));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(Logger::default().exclude("/health"))
            .wrap(Compress::default())
            .wrap(
                CookieSession::private(secret_key.as_bytes())
                    .domain(public_host.domain().expect("public_host.domain()"))
                    .name("enseada_session")
                    .path("/")
                    .secure(tls.enabled())
                    .http_only(true)
                    .same_site(SameSite::Strict),
            )
            .wrap(Cors::default())
            .wrap(default_headers(cfg))
            // (matteojoliveau) This requires to inject the Enforcer as `Data<RwLock<Enforcer>>`
            // because app_data() stuff is not accessible in nested scopes.
            // Should hopefully be fixed with Actix Web 3.0
            .data(enforcer.clone())
            .configure(user::mount(
                couch.database(crate::couchdb::name::USERS, true),
                event_bus.clone(),
            ))
            .configure(crate::rbac::mount)
            .configure(crate::oauth::mount(
                couch.database(crate::couchdb::name::OAUTH, true),
            ))
            .configure(observability::mount)
            .configure(oci::mount(
                cfg,
                couch.database(crate::couchdb::name::OCI, true),
                event_bus.clone(),
            ))
            .configure(routes::mount)
            .configure(dashboard::mount)
            .default_service(dashboard::default_service())
    });

    let server = if let Some(host) = public_host.host() {
        server.server_hostname(host.to_string())
    } else {
        server
    };

    let server = if tls.enabled() {
        let mut config = ServerConfig::new(NoClientAuth::new());
        let cert_f = &mut File::open(tls.cert_path().expect("missing tls.cert.path"))?;
        let key_f = &mut File::open(tls.key_path().expect("missing tls.key.path"))?;
        let certs = get_certs(cert_f);
        let key = get_rsa_key(key_f);
        config.set_single_cert(certs, key).unwrap();
        server.bind_rustls(&address, config)
    } else {
        server.bind(&address)
    }?;

    log::info!("Server started listening on {}", &address);
    server.run().await?;
    watcher.stop();

    Ok(())
}

fn get_certs(cert: &File) -> Vec<Certificate> {
    let buf = &mut BufReader::new(cert);
    certs(buf).unwrap()
}

fn get_rsa_key(key: &mut File) -> PrivateKey {
    let rsa_buf = &mut BufReader::new(key.try_clone().unwrap());
    let pkcs_buf = &mut BufReader::new(key.try_clone().unwrap());
    let rsa = rsa_private_keys(rsa_buf).unwrap();
    key.seek(SeekFrom::Start(0)).unwrap();
    let pkcs8 = pkcs8_private_keys(pkcs_buf).unwrap();
    rsa.first()
        .or_else(|| pkcs8.first())
        .expect("key format not supported. must be either RSA or PKCS8-encoded.")
        .clone()
}

fn default_headers(cfg: &Configuration) -> DefaultHeaders {
    let h = DefaultHeaders::new().header("Server", "enseada");

    if cfg.tls().enabled() {
        h.header(
            "Strict-Transport-Security",
            "max-age=31536000;includeSubDomains",
        )
    } else {
        h
    }
}
