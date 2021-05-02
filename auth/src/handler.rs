use url::Url;

use api::Resource;
use controller_runtime::{Duration, ResourceManager, Utc};
use oauth::async_trait;
use oauth::client::{Client, ClientKind};
use oauth::error::{Error, ErrorKind};
use oauth::handler::{BasicAuth, RequestHandler};
use oauth::request::{AuthorizationCodeRequest, AuthorizationRequest, IntrospectionRequest, RefreshTokenRequest, RevocationRequest};
use oauth::response::{AuthorizationResponse, IntrospectionResponse, RevocationResponse, TokenResponse};
use oauth::scope::Scope;
use oauth::session::Session;

use crate::api::v1alpha1::{OAuthAuthorizationCode, OAuthClient};
use api::core::v1alpha1::Metadata;
use std::ops::Add;
use oauth::code::AuthorizationCode;

#[derive(Clone)]
pub struct OAuthHandler {
    client_manager: ResourceManager<OAuthClient>,
    auth_code_manager: ResourceManager<OAuthAuthorizationCode>,
    secret_key: String,
}

impl OAuthHandler {
    pub fn new(client_manager: ResourceManager<OAuthClient>, auth_code_manager: ResourceManager<OAuthAuthorizationCode>, secret_key: String) -> Self {
        OAuthHandler { client_manager, auth_code_manager, secret_key }
    }

    async fn get_client(&self, client_id: &str) -> oauth::Result<OAuthClient> {
        self.client_manager.find(client_id).await
            .map_err(|err| Error::new(ErrorKind::ServerError, err))?
            .ok_or_else(|| Error::new(ErrorKind::InvalidClient, "invalid client_id".to_string()))
    }

    async fn validate_client(&self, client_id: &str, redirect_uri: Option<&str>, scope: &Scope) -> oauth::Result<Client> {
        let client = self.get_client(client_id).await?;

        if !client.spec.allowed_scopes.is_superset(scope) {
            return Err(Error::new(
                ErrorKind::InvalidScope,
                "invalid scopes".to_string(),
            ));
        }

        if let Some(redirect_uri) = redirect_uri {
            let uri = Url::parse(&redirect_uri)
                .map_err(|err| Error::new(ErrorKind::InvalidRedirectUri, err.to_string()))?;

            if !client.spec.allowed_redirect_uris.contains(&uri) {
                return Err(Error::new(
                    ErrorKind::InvalidRedirectUri,
                    String::from("invalid redirect URI"),
                ));
            }
        }

        Ok(client.into())
    }

    async fn authenticate_client(
        &self,
        client: &Client,
        client_secret: Option<&str>,
    ) -> oauth::Result<()> {
        match client.kind() {
            ClientKind::Public => Ok(()),
            ClientKind::Confidential=> {
                let client = self.get_client(client.client_id()).await?;
                // TODO
                match client_secret {
                    Some(client_secret) => {
                        if crypto::verify_password("", client_secret)? {
                            Ok(())
                        } else {
                            Err(Error::new(
                                ErrorKind::InvalidClient,
                                "invalid client credentials".to_string(),
                            ))
                        }
                    }
                    None => Err(Error::new(
                        ErrorKind::InvalidClient,
                        "client credentials required".to_string(),
                    )),
                }
            }
        }
    }
}

#[async_trait]
impl RequestHandler<AuthorizationRequest, AuthorizationResponse> for OAuthHandler {
    async fn validate(&self, req: &AuthorizationRequest, client_auth: Option<&BasicAuth>) -> oauth::Result<Client> {
        self.validate_client(&req.client_id, Some(&req.redirect_uri), &req.scope)
            .await
            .and_then(|client| {
                if let Some(pkce) = &req.pkce {
                    pkce.validate(req.state.as_deref())?;
                }
                Ok(client)
            })
            .map_err(|mut err| {
                err.set_state(req.state.as_deref());
                err
            })
    }

    async fn handle(&self, req: &AuthorizationRequest, session: &mut Session) -> oauth::Result<AuthorizationResponse> {
        session.set_scope(req.scope.clone());

        let secret = crypto::generate_token(16).unwrap();
        let code_hash = crypto::generate_signature(secret.to_string().as_str(), &self.secret_key);
        let code = OAuthAuthorizationCode {
            type_meta: OAuthAuthorizationCode::type_meta(),
            metadata: Metadata::named(&code_hash),
            session: session.clone(),
            expiration: Utc::now().add(Duration::minutes(5)),
            pkce: req.pkce.clone(),
            code_hash,
        };
        let code = self.auth_code_manager.put(&code.name().to_string(), code).await.map_err(|err| Error::new(ErrorKind::ServerError, err))?;
        let code = AuthorizationCode::new(secret, code.session, code.expiration, code.pkce);
        let res = AuthorizationResponse::new(code, req.state.clone());
        Ok(res)
    }
}

#[async_trait]
impl RequestHandler<AuthorizationCodeRequest, TokenResponse> for OAuthHandler {
    async fn validate(&self, req: &AuthorizationCodeRequest, client_auth: Option<&BasicAuth>) -> oauth::Result<Client> {
        todo!()
    }

    async fn handle(&self, req: &AuthorizationCodeRequest, session: &mut Session) -> oauth::Result<TokenResponse> {
        todo!()
    }
}

#[async_trait]
impl RequestHandler<RefreshTokenRequest, TokenResponse> for OAuthHandler {
    async fn validate(&self, req: &RefreshTokenRequest, client_auth: Option<&BasicAuth>) -> oauth::Result<Client> {
        todo!()
    }

    async fn handle(&self, req: &RefreshTokenRequest, session: &mut Session) -> oauth::Result<TokenResponse> {
        todo!()
    }
}

#[async_trait]
impl RequestHandler<IntrospectionRequest, IntrospectionResponse> for OAuthHandler {
    async fn validate(&self, req: &IntrospectionRequest, client_auth: Option<&BasicAuth>) -> oauth::Result<Client> {
        todo!()
    }

    async fn handle(&self, req: &IntrospectionRequest, session: &mut Session) -> oauth::Result<IntrospectionResponse> {
        todo!()
    }
}

#[async_trait]
impl RequestHandler<RevocationRequest, RevocationResponse> for OAuthHandler {
    async fn validate(&self, req: &RevocationRequest, client_auth: Option<&BasicAuth>) -> oauth::Result<Client> {
        todo!()
    }

    async fn handle(&self, req: &RevocationRequest, session: &mut Session) -> oauth::Result<RevocationResponse> {
        todo!()
    }
}
