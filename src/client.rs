use std::io;

use reqwest::{header::HeaderValue, Body, Method, Request, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot connect to server")]
    Connection(#[from] io::Error),
    #[error("HTTP Client error")]
    HttpClient(#[from] reqwest::Error),
    #[error("Invalid token received")]
    InvalidTokenValue,
    #[error("Invalid URL")]
    InvalidURL(#[from] url::ParseError),
    #[error("Failed to parse json")]
    ParseError(#[from] serde_json::Error),
    #[error("Error response: [{0}] {1}")]
    ErrorResponse(u16, String),
}

pub struct Client {
    base_url: Url,
    token: Option<String>,
    http_client: reqwest::Client,
}

impl Client {
    pub async fn new_with_token(base_url: String, token: Option<String>) -> Result<Self, Error> {
        let url = url::Url::parse(&base_url)?;

        Ok(Client {
            base_url: url,
            token,
            http_client: reqwest::Client::new(),
        })
    }

    pub async fn request(&self, req: reqwest::Request) -> Result<reqwest::Response, Error> {
        Ok(self.http_client.execute(req).await?)
    }

    pub(crate) fn new_request<S: AsRef<str>>(
        &self,
        method: reqwest::Method,
        path: S,
        body: Option<Body>,
    ) -> Result<reqwest::Request, Error> {
        self.new_request_inner(method, path.as_ref(), body)
    }

    fn new_request_inner(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Body>,
    ) -> Result<reqwest::Request, Error> {
        let mut req = Request::new(method, self.base_url.join(path)?);

        let mut header_value = HeaderValue::from_str(&format!(
            "Bearer {}",
            self.token.as_ref().unwrap_or(&String::from("anonymous"))
        ))
        .map_err(|_| Error::InvalidTokenValue)?;
        header_value.set_sensitive(true);

        req.headers_mut().insert("Authorization", header_value);

        if let &Method::PATCH = req.method() {
            req.headers_mut().insert(
                "Content-Type",
                HeaderValue::from_static("application/json-patch+json"),
            );
        } else {
            req.headers_mut()
                .insert("Content-Type", HeaderValue::from_static("application/json"));
        }

        *req.body_mut() = body;

        Ok(req)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorMessage {
    message: String,
}

pub async fn status_unwrap(resp: Response) -> Result<Response, Error> {
    match resp.status().as_u16() {
        code if code < 200 || code >= 300 => {
            let err_msg = resp.json::<ErrorMessage>().await?;

            Err(Error::ErrorResponse(code, err_msg.message))
        }
        _ => Ok(resp),
    }
}
