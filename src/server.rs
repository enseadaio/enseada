use io::BufReader;
use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom};
use std::sync::Arc;

use actix_session::CookieSession;
use actix_web::{App, HttpServer};
use actix_web::cookie::SameSite;
use actix_web::middleware::{DefaultHeaders, Logger, NormalizePath};
use actix_web::web::Data;
use rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig};
use rustls::internal::pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use tokio::sync::RwLock;
use url::Url;

use crate::config::CONFIG;
use crate::containers;
use crate::couchdb::{self, add_couch_client};
use crate::http::handler::oauth;
use crate::http::handler::user::add_user_service;
use crate::rbac::Enforcer;
use crate::rbac::watcher::Watcher;
use crate::routes::routes;

pub async fn run() -> io::Result<()> {
    let address = format!("0.0.0.0:{}", CONFIG.port());
    let public_host: &Url = CONFIG.public_host();
    let secret_key = CONFIG.secret_key();
    let tls = CONFIG.tls();

    let rbac_db = Arc::new(couchdb::SINGLETON.database(couchdb::db::name::RBAC, true));
    let mut enforcer = Enforcer::new(rbac_db.clone());
    enforcer.load_rules().await.expect("enforcer.load_rules()");
    let enforcer = Data::new(RwLock::new(enforcer));
    let watcher = Watcher::new(rbac_db.clone(), enforcer.clone().into_inner());
    watcher.start().expect("watcher.start()");

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(NormalizePath)
            .wrap(CookieSession::private(secret_key.as_bytes())
                .domain(public_host.domain().expect("public_host.domain()"))
                .name("enseada_session")
                .path("/")
                .secure(tls.enabled())
                .http_only(true)
                .same_site(SameSite::Strict))
            .wrap(default_headers())
            .app_data(enforcer.clone())
            .configure(add_couch_client)
            .configure(add_user_service)
            .configure(oauth::add_oauth_handler)
            .configure(containers::routes)
            .configure(routes)
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
    rsa.first().or_else(|| pkcs8.first())
        .expect("key format not supported. must be either RSA or PKCS8-encoded.")
        .clone()
}

fn default_headers() -> DefaultHeaders {
    let h = DefaultHeaders::new()
        .header("Server", "Enseada");

    if CONFIG.tls().enabled() {
        h.header("Strict-Transport-Security", "max-age=31536000;includeSubDomains")
    } else {
        h
    }
}
