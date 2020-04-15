use std::str::FromStr;

use actix_web::dev::RequestHead;
use actix_web::guard::Guard;
use actix_web::http::{header, Uri};

pub struct SubdomainGuard(String);

impl Guard for SubdomainGuard {
    fn check(&self, req: &RequestHead) -> bool {
        let uri = req.headers
            .get(header::HOST)
            .and_then(|host_value| host_value.to_str().ok())
            .or_else(|| req.uri.host())
            .map(|host: &str| Uri::from_str(host).ok())
            .and_then(|host_success| host_success);

        let req_host_uri = if let Some(uri) = uri {
            uri
        } else {
            return false;
        };

        match req_host_uri.host() {
            Some(uri_host) => uri_host.starts_with(&self.0),
            None => false
        }
    }
}

pub fn Subdomain<H: AsRef<str>>(prefix: H) -> SubdomainGuard {
    SubdomainGuard(prefix.as_ref().to_string())
}