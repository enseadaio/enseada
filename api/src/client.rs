use std::str::FromStr;

use http::Uri;
use tonic::transport::{Channel, Endpoint};

use crate::error::Error;
use crate::tls::TlsConfig;
use crate::users::v1alpha1::users_client::UsersClient;

const USER_AGENT: &'static str = "enseada";

#[derive(Clone)]
pub struct Client {
    channel: Channel,
}

impl Client {
    pub fn new(address: impl AsRef<[u8]> + 'static, tls: Option<TlsConfig>) -> Result<Self, Error> {
        let uri = Uri::from_maybe_shared(address)?;
        let endpoint = Endpoint::from(uri)
            .user_agent(USER_AGENT)?;

        let endpoint = if let Some(cfg) = tls.map(TlsConfig::into) {
            endpoint.tls_config(cfg)?
        } else {
            endpoint
        };

        Ok(Self {
            channel: endpoint.connect_lazy()?,
        })
    }

    pub fn users(&self) -> UsersClient<Channel> {
        UsersClient::new(self.channel.clone())
    }
}
