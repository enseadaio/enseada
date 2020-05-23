use std::str::FromStr;

use actix_web::dev::RequestHead;
use actix_web::guard::Guard;
use actix_web::http::{header, Uri};

pub struct SubdomainGuard(String);

impl Guard for SubdomainGuard {
    fn check(&self, req: &RequestHead) -> bool {
        log::trace!("Guard checking for subdomain {}", &self.0);
        let uri = req.headers
            .get(header::HOST)
            .and_then(|host_value| host_value.to_str().ok())
            .or_else(|| req.uri.host())
            .map(|host: &str| Uri::from_str(host).ok())
            .and_then(|host_success| host_success);

        log::trace!("Checking URI {:?}", &uri);

        let req_host_uri = if let Some(uri) = uri {
            uri
        } else {
            return false;
        };

        match req_host_uri.host() {
            Some(uri_host) => {
                let matches = uri_host.starts_with(&self.0);
                log::trace!("URI {} starts with {}: {}", &uri_host, &self.0, matches);
                matches
            },
            None => false
        }
    }
}

pub fn subdomain<H: AsRef<str>>(prefix: H) -> SubdomainGuard {
    let mut prefix = prefix.as_ref().to_string();
    if !prefix.ends_with('.') {
        prefix.push('.');
    }
    SubdomainGuard(prefix)
}