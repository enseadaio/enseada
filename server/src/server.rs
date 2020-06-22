use io::BufReader;
use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom};
use std::sync::Arc;

use actix_session::CookieSession;
use actix_web::cookie::SameSite;
use actix_web::middleware::errhandlers::ErrorHandlers;
use actix_web::middleware::{DefaultHeaders, Logger, NormalizePath};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use http::StatusCode;
use rustls::internal::pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig};
use tokio::sync::RwLock;
use url::Url;

use crate::config::CONFIG;
use crate::couchdb::{name as dbname, SINGLETON};
use crate::http::error;
use crate::rbac::watcher::Watcher;
use crate::rbac::Enforcer;
use crate::{dashboard, oauth, observability, oci, rbac, routes, user};

pub async fn run() -> io::Result<()> {
    let address = format!("0.0.0.0:{}", CONFIG.port());
    let public_host: &Url = CONFIG.public_host();
    let secret_key = CONFIG.secret_key();
    let tls = CONFIG.tls();

    let rbac_db = Arc::new(SINGLETON.database(dbname::RBAC, true));
    let mut enforcer = Enforcer::new(rbac_db.clone());
    enforcer.load_rules().await.expect("enforcer.load_rules()");
    let enforcer = Data::new(RwLock::new(enforcer));
    let watcher = Watcher::new(rbac_db.clone(), enforcer.clone().into_inner());
    watcher.start().expect("watcher.start()");

    let server = HttpServer::new(move || {
        App::new()
            .wrap(NormalizePath)
            .wrap(Logger::default().exclude("/health"))
            .wrap(
                CookieSession::private(secret_key.as_bytes())
                    .domain(public_host.domain().expect("public_host.domain()"))
                    .name("enseada_session")
                    .path("/")
                    .secure(tls.enabled())
                    .http_only(true)
                    .same_site(SameSite::Strict),
            )
            .wrap(ErrorHandlers::new().handler(StatusCode::BAD_REQUEST, error::handle_bad_request))
            .wrap(
                ErrorHandlers::new().handler(StatusCode::UNAUTHORIZED, error::handle_unauthorized),
            )
            .wrap(default_headers())
            .app_data(enforcer.clone())
            .configure(user::mount)
            .configure(rbac::mount)
            .configure(oauth::mount)
            .configure(dashboard::mount)
            .configure(observability::mount)
            .configure(oci::mount)
            .configure(routes::mount)
    });

    let server = if let Some(host) = public_host.host() {
        server.server_hostname(host.to_string())
    } else {
        server
    };

    let server = if tls.enabled() {
        let mut config = ServerConfig::new(NoClientAuth::new());
        let cert_f = &mut File::open(tls.cert_path().expect("missing tls.cert_path"))?;
        let key_f = &mut File::open(tls.key_path().expect("missing tls.cert_path"))?;
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

fn default_headers() -> DefaultHeaders {
    let h = DefaultHeaders::new().header("Server", "Enseada");

    if CONFIG.tls().enabled() {
        h.header(
            "Strict-Transport-Security",
            "max-age=31536000;includeSubDomains",
        )
    } else {
        h
    }
}
