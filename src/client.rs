use std::{io, pin::Pin, time::Duration};

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::{header::HeaderValue, Body, Method, Request, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::{
    Change, Commit, CommitMessage, Entry, Project, PushResult, Query, Repository, Revision,
    WatchResult,
};

const WATCH_BUFFER_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot connect to server")]
    Connection(#[from] io::Error),
    #[error("HTTP Client error")]
    HttpClient(#[from] reqwest::Error),
    #[error("Invalid token received")]
    InvalidTokenValue,
    #[allow(clippy::upper_case_acronyms)]
    #[error("Invalid URL")]
    InvalidURL(#[from] url::ParseError),
    #[error("Failed to parse json")]
    ParseError(#[from] serde_json::Error),
    #[error("Invalid params: {0}")]
    InvalidParams(&'static str),
    #[error("Error response: [{0}] {1}")]
    ErrorResponse(u16, String),
}

#[derive(Clone)]
pub struct Client {
    base_url: Url,
    token: HeaderValue,
    http_client: reqwest::Client,
}

impl Client {
    pub async fn from_token(base_url: &str, token: Option<&str>) -> Result<Self, Error> {
        let url = url::Url::parse(&base_url)?;
        let http_client = reqwest::Client::builder().user_agent("cd-rs").build()?;

        let mut header_value = HeaderValue::from_str(&format!(
            "Bearer {}",
            token.as_ref().unwrap_or(&"anonymous")
        ))
        .map_err(|_| Error::InvalidTokenValue)?;
        header_value.set_sensitive(true);

        Ok(Client {
            base_url: url,
            token: header_value,
            http_client,
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

    pub fn project<'a>(&'a self, project_name: &'a str) -> ProjectClient<'a> {
        ProjectClient {
            client: self,
            project: project_name,
        }
    }

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
pub async fn status_unwrap(resp: Response) -> Result<Response, Error> {
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

#[async_trait]
pub trait ProjectService {
    async fn list_projects(&self) -> Result<Vec<Project>, Error>;
    async fn list_removed_projects(&self) -> Result<Vec<String>, Error>;
    async fn create_project(&self, project_name: &str) -> Result<Project, Error>;
    async fn remove_project(&self, project_name: &str) -> Result<(), Error>;
    async fn unremove_project(&self, project_name: &str) -> Result<Project, Error>;
    async fn purge_project(&self, project_name: &str) -> Result<(), Error>;
}

#[async_trait]
impl ProjectService for Client {
    async fn list_projects(&self) -> Result<Vec<Project>, Error> {
        crate::services::project::list(self).await
    }

    async fn list_removed_projects(&self) -> Result<Vec<String>, Error> {
        crate::services::project::list_removed(self).await
    }

    async fn create_project(&self, project_name: &str) -> Result<Project, Error> {
        crate::services::project::create(self, project_name).await
    }

    async fn remove_project(&self, project_name: &str) -> Result<(), Error> {
        crate::services::project::remove(self, project_name).await
    }

    async fn unremove_project(&self, project_name: &str) -> Result<Project, Error> {
        crate::services::project::unremove(self, project_name).await
    }

    async fn purge_project(&self, project_name: &str) -> Result<(), Error> {
        crate::services::project::purge(self, project_name).await
    }
}

#[async_trait]
pub trait RepoService {
    async fn list_repos(&self) -> Result<Vec<Repository>, Error>;
    async fn list_removed_repos(&self) -> Result<Vec<Repository>, Error>;
    async fn create_repo(&self, repo_name: &str) -> Result<Repository, Error>;
    async fn remove_repo(&self, repo_name: &str) -> Result<(), Error>;
    async fn unremove_repo(&self, repo_name: &str) -> Result<Repository, Error>;
    async fn purge_repo(&self, repo_name: &str) -> Result<(), Error>;
}

pub struct ProjectClient<'a> {
    client: &'a Client,
    project: &'a str,
}

#[async_trait]
impl<'a> RepoService for ProjectClient<'a> {
    async fn list_repos(&self) -> Result<Vec<Repository>, Error> {
        crate::services::repository::list_by_project_name(self.client, self.project).await
    }

    async fn list_removed_repos(&self) -> Result<Vec<Repository>, Error> {
        crate::services::repository::list_removed_by_project_name(self.client, self.project).await
    }

    async fn create_repo(&self, repo_name: &str) -> Result<Repository, Error> {
        crate::services::repository::create(self.client, self.project, repo_name).await
    }

    async fn remove_repo(&self, repo_name: &str) -> Result<(), Error> {
        crate::services::repository::remove(self.client, self.project, repo_name).await
    }

    async fn unremove_repo(&self, repo_name: &str) -> Result<Repository, Error> {
        crate::services::repository::unremove(self.client, self.project, repo_name).await
    }

    async fn purge_repo(&self, repo_name: &str) -> Result<(), Error> {
        crate::services::repository::purge(self.client, self.project, repo_name).await
    }
}

pub struct RepoClient<'a> {
    client: &'a Client,
    project: &'a str,
    repo: &'a str,
}

#[async_trait]
pub trait ContentService {
    async fn get_file(&self, revision: Revision, query: &Query) -> Result<Entry, Error>;
    async fn get_files(&self, revision: Revision, path_pattern: &str) -> Result<Vec<Entry>, Error>;
    async fn list_files(&self, revision: Revision, path_pattern: &str)
        -> Result<Vec<Entry>, Error>;
    async fn get_diff(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        query: &Query,
    ) -> Result<Change, Error>;
    async fn get_diffs(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        path_pattern: &str,
    ) -> Result<Vec<Change>, Error>;
    async fn get_history(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        path: &str,
        max_commits: u32,
    ) -> Result<Vec<Commit>, Error>;
    async fn push(
        &self,
        base_revision: Revision,
        cm: CommitMessage,
        changes: Vec<Change>,
    ) -> Result<PushResult, Error>;
}

#[async_trait]
impl<'a> ContentService for RepoClient<'a> {
    async fn get_file(&self, revision: Revision, query: &Query) -> Result<Entry, Error> {
        crate::services::content::get_file(self.client, self.project, self.repo, revision, query)
            .await
    }

    async fn get_files(&self, revision: Revision, path_pattern: &str) -> Result<Vec<Entry>, Error> {
        crate::services::content::get_files(
            self.client,
            self.project,
            self.repo,
            revision,
            path_pattern,
        )
        .await
    }

    async fn list_files(
        &self,
        revision: Revision,
        path_pattern: &str,
    ) -> Result<Vec<Entry>, Error> {
        crate::services::content::list_files(
            self.client,
            self.project,
            self.repo,
            revision,
            path_pattern,
        )
        .await
    }

    async fn get_diff(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        query: &Query,
    ) -> Result<Change, Error> {
        crate::services::content::get_diff(
            self.client,
            self.project,
            self.repo,
            from_rev,
            to_rev,
            query,
        )
        .await
    }

    async fn get_diffs(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        path_pattern: &str,
    ) -> Result<Vec<Change>, Error> {
        crate::services::content::get_diffs(
            self.client,
            self.project,
            self.repo,
            from_rev,
            to_rev,
            path_pattern,
        )
        .await
    }

    async fn get_history(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        path: &str,
        max_commits: u32,
    ) -> Result<Vec<Commit>, Error> {
        crate::services::content::get_history(
            self.client,
            self.project,
            self.repo,
            from_rev,
            to_rev,
            path,
            max_commits,
        )
        .await
    }

    async fn push(
        &self,
        base_revision: Revision,
        cm: CommitMessage,
        changes: Vec<Change>,
    ) -> Result<PushResult, Error> {
        crate::services::content::push(
            self.client,
            self.project,
            self.repo,
            base_revision,
            cm,
            changes,
        )
        .await
    }
}

pub trait WatchService {
    fn watch_file_stream(
        &self,
        query: &Query,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchResult> + Send>>, Error>;

    fn watch_repo_stream(
        &self,
        path_pattern: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchResult> + Send>>, Error>;
}

impl<'a> WatchService for RepoClient<'a> {
    fn watch_file_stream(
        &self,
        query: &Query,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchResult> + Send>>, Error> {
        Ok(crate::services::watch::watch_file_stream(
            self.client.clone(),
            self.project,
            self.repo,
            query,
        )?
        .boxed())
    }

    fn watch_repo_stream(
        &self,
        path_pattern: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchResult> + Send>>, Error> {
        Ok(crate::services::watch::watch_repo_stream(
            self.client.clone(),
            self.project,
            self.repo,
            path_pattern,
        )?
        .boxed())
    }
}
