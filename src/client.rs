use std::io;

use reqwest::{header::HeaderValue, Body, Request};
use thiserror::Error;
use url::Url;
use yup_oauth2::{authenticator::DefaultAuthenticator, ServiceAccountKey};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot connect to server")]
    Connection(#[from] io::Error),
    #[error("Failed to create authenticator")]
    Authenticator,
    #[error("Failed to authenticate")]
    Authenticate(#[from] yup_oauth2::Error),
    #[error("HTTP Client error")]
    HttpClient(#[from] reqwest::Error),
    #[error("Invalid token received")]
    InvalidTokenValue,
    #[error("Invalid URL")]
    InvalidURL(#[from] url::ParseError),
    #[error("Error response: [{0}] {1}")]
    ErrorResponse(u16, String),
}

const LOGIN_PATH: &str = "api/v1/login";

pub struct Client {
    base_url: Url,
    authenticator: DefaultAuthenticator,
    http_client: reqwest::Client,
}

impl Client {
    pub async fn new_with_token(base_url: String, token: String) -> Result<Self, Error> {
        let url = url::Url::parse(&base_url)?;

        let service_account_key = ServiceAccountKey {
            key_type: None,
            project_id: None,
            private_key_id: None,
            private_key: token,
            client_email: String::from(""),
            client_id: None,
            auth_uri: None,
            token_uri: format!("{}/{}", &base_url, LOGIN_PATH),
            auth_provider_x509_cert_url: None,
            client_x509_cert_url: None,
        };

        let authenticator = yup_oauth2::ServiceAccountAuthenticator::builder(service_account_key)
            .build()
            .await
            .map_err(|_| Error::Authenticator)?;

        Ok(Client {
            base_url: url,
            authenticator,
            http_client: reqwest::Client::new(),
        })
    }

    pub async fn request(&self, mut req: reqwest::Request) -> Result<reqwest::Response, Error> {
        let token = self.authenticator.token::<&str>(&[]).await?;

        let mut header_value = HeaderValue::from_str(&format!("Bearer {}", token.as_str()))
            .map_err(|_| Error::InvalidTokenValue)?;
        header_value.set_sensitive(true);

        req.headers_mut().insert("Authentication", header_value);

        Ok(self.http_client.execute(req).await?)
    }

    pub fn new_request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Body>,
    ) -> Result<reqwest::Request, Error> {
        let mut req = Request::new(method, self.base_url.join(path)?);
        *req.body_mut() = body;

        Ok(req)
    }
}
