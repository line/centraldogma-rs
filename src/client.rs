use std::time::Duration;

use reqwest::{header::HeaderValue, Body, Method, Request, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::model::Revision;

const WATCH_BUFFER_TIMEOUT: Duration = Duration::from_secs(5);

/// An error happen with the client.
/// Errors that can occur include I/O and parsing errors,
/// as well as error response from centraldogma server
#[derive(Error, Debug)]
pub enum Error {
    /// Error from HTTP Request
    #[error("HTTP Client error")]
    HttpClient(#[from] reqwest::Error),

    /// Error when provided invalid base_url
    #[allow(clippy::upper_case_acronyms)]
    #[error("Invalid URL")]
    InvalidURL(#[from] url::ParseError),

    /// Error when parse response json into Rust model structs
    #[error("Failed to parse json")]
    ParseError(#[from] serde_json::Error),

    /// Error when provided invalid parameters
    #[error("Invalid params: {0}")]
    InvalidParams(&'static str),

    /// Errors returned from CentralDomgma server (status code > 300)  
    /// (HTTP StatusCode, Response string from server)
    #[error("Error response: [{0}] {1}")]
    ErrorResponse(u16, String),
}

/// Root client for top level APIs.  
/// Implements [`crate::ProjectService`]
#[derive(Clone)]
pub struct Client {
    base_url: Url,
    token: HeaderValue,
    http_client: reqwest::Client,
}

impl Client {
    /// Returns a new client from provided `base_url` and an optional
    /// `token` string for authentication.
    /// Only visible ASCII characters (32-127) are permitted as token.
    pub async fn new(base_url: &str, token: Option<&str>) -> Result<Self, Error> {
        let url = url::Url::parse(&base_url)?;
        let http_client = reqwest::Client::builder().user_agent("cd-rs").build()?;

        let mut header_value = HeaderValue::from_str(&format!(
            "Bearer {}",
            token.as_ref().unwrap_or(&"anonymous")
        ))
        .map_err(|_| Error::InvalidParams("Invalid token received"))?;
        header_value.set_sensitive(true);

        Ok(Client {
            base_url: url,
            token: header_value,
            http_client,
        })
    }

    pub(crate) async fn request(&self, req: reqwest::Request) -> Result<reqwest::Response, Error> {
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

        // HeaderValue's clone is cheap as it's using Bytes underneath
        req.headers_mut()
            .insert("Authorization", self.token.clone());

        if let Method::PATCH = *req.method() {
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

    pub(crate) fn new_watch_request<S: AsRef<str>>(
        &self,
        method: reqwest::Method,
        path: S,
        body: Option<Body>,
        last_known_revision: Option<Revision>,
        timeout: Duration,
    ) -> Result<reqwest::Request, Error> {
        let mut req = self.new_request(method, path, body)?;

        match last_known_revision {
            Some(rev) => {
                let val = HeaderValue::from_str(&rev.to_string()).unwrap();
                req.headers_mut().insert("if-none-match", val);
            }
            None => {
                let val = HeaderValue::from_str(&Revision::HEAD.to_string()).unwrap();
                req.headers_mut().insert("if-none-match", val);
            }
        }

        if timeout.as_secs() != 0 {
            let val = HeaderValue::from_str(&format!("wait={}", timeout.as_secs())).unwrap();
            req.headers_mut().insert("prefer", val);
        }

        let req_timeout = timeout.checked_add(WATCH_BUFFER_TIMEOUT).unwrap();
        req.timeout_mut().replace(req_timeout);

        Ok(req)
    }

    /// Creates a temporary client within a context of the specified Project.
    pub fn project<'a>(&'a self, project_name: &'a str) -> ProjectClient<'a> {
        ProjectClient {
            client: self,
            project: project_name,
        }
    }

    /// Creates a temporary client within a context of the specified Repository.
    pub fn repo<'a>(&'a self, project_name: &'a str, repo_name: &'a str) -> RepoClient<'a> {
        RepoClient {
            client: self,
            project: project_name,
            repo: repo_name,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorMessage {
    message: String,
}

/// convert HTTP Response with status < 200 and > 300 to Error
pub(crate) async fn status_unwrap(resp: Response) -> Result<Response, Error> {
    match resp.status().as_u16() {
        code if !(200..300).contains(&code) => {
            let err_body = resp.text().await?;
            let err_msg: ErrorMessage =
                serde_json::from_str(&err_body).unwrap_or(ErrorMessage { message: err_body });

            Err(Error::ErrorResponse(code, err_msg.message))
        }
        _ => Ok(resp),
    }
}

/// A temporary client within context of a project.  
/// Created by [`Client::project()`]  
/// Implements [`crate::RepoService`]
pub struct ProjectClient<'a> {
    pub(crate) client: &'a Client,
    pub(crate) project: &'a str,
}

/// A temporary client within context of a Repository.  
/// Created by [`Client::repo()`]  
/// Implements [`crate::ContentService`] and
/// [`crate::WatchService`]
pub struct RepoClient<'a> {
    pub(crate) client: &'a Client,
    pub(crate) project: &'a str,
    pub(crate) repo: &'a str,
}
