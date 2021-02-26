use std::{pin::Pin, time::Duration};

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::{header::HeaderValue, Body, Method, Request, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::model::{
    Change, Commit, CommitMessage, Entry, ListEntry, Project, PushResult, Query, Repository,
    Revision, WatchFileResult, WatchRepoResult,
};

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

/// Root client for top level APIs
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

/// Project-related APIs
#[async_trait]
pub trait ProjectService {
    /// Retrieves the list of the projects.
    async fn list_projects(&self) -> Result<Vec<Project>, Error>;

    /// Retrieves the list of the removed projects,
    /// which can be [unremoved](#tymethod.unremove_project)
    /// or [purged](#tymethod.purge_project).
    async fn list_removed_projects(&self) -> Result<Vec<String>, Error>;

    /// Creates a project.
    async fn create_project(&self, project_name: &str) -> Result<Project, Error>;

    /// Removes a project. A removed project can be [unremoved](#tymethod.unremove_project).
    async fn remove_project(&self, project_name: &str) -> Result<(), Error>;

    /// Unremoves a project.
    async fn unremove_project(&self, project_name: &str) -> Result<Project, Error>;

    /// Purges a project that was removed before.
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

/// Repository-related APIs
#[async_trait]
pub trait RepoService {
    /// Retrieves the list of the repositories.
    async fn list_repos(&self) -> Result<Vec<Repository>, Error>;

    /// Retrieves the list of the removed repositories, which can be
    /// [unremoved](#tymethod.unremove_repo).
    async fn list_removed_repos(&self) -> Result<Vec<Repository>, Error>;

    /// Creates a repository.
    async fn create_repo(&self, repo_name: &str) -> Result<Repository, Error>;

    /// Removes a repository, removed repository can be
    /// [unremoved](#tymethod.unremove_repo).
    async fn remove_repo(&self, repo_name: &str) -> Result<(), Error>;

    /// Unremoves a repository.
    async fn unremove_repo(&self, repo_name: &str) -> Result<Repository, Error>;

    /// Purges a repository that was removed before.
    async fn purge_repo(&self, repo_name: &str) -> Result<(), Error>;
}

/// A temporary client within context of a project.
/// Created by [`Client::project()`]
/// Implemts [`RepoService`]
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

/// A temporary client within context of a Repository.  
/// Created by [`Client::repo()`]  
/// Implements [`ContentService`]
pub struct RepoClient<'a> {
    client: &'a Client,
    project: &'a str,
    repo: &'a str,
}

/// Content-related APIs
#[async_trait]
pub trait ContentService {
    /// Queries a file at the specified [`Revision`] and path with the specified [`Query`].
    async fn get_file(&self, revision: Revision, query: &Query) -> Result<Entry, Error>;

    /// Retrieves the files at the specified [`Revision`] matched by the path pattern.
    ///
    /// A path pattern is a variant of glob:
    ///   * `"/**"` - find all files recursively
    ///   * `"*.json"` - find all JSON files recursively
    ///   * `"/foo/*.json"` - find all JSON files under the directory /foo
    ///   * `"/*/foo.txt"` - find all files named foo.txt at the second depth level
    ///   * `"*.json,/bar/*.txt"` - use comma to specify more than one pattern.
    ///   A file will be matched if any pattern matches.
    async fn get_files(&self, revision: Revision, path_pattern: &str) -> Result<Vec<Entry>, Error>;

    /// Retrieves the list of the files at the specified [`Revision`] matched by the path pattern.
    ///
    /// A path pattern is a variant of glob:
    ///   * `"/**"` - find all files recursively
    ///   * `"*.json"` - find all JSON files recursively
    ///   * `"/foo/*.json"` - find all JSON files under the directory /foo
    ///   * `"/*/foo.txt"` - find all files named foo.txt at the second depth level
    ///   * `"*.json,/bar/*.txt"` - use comma to specify more than one pattern.
    ///   A file will be matched if any pattern matches.
    async fn list_files(
        &self,
        revision: Revision,
        path_pattern: &str,
    ) -> Result<Vec<ListEntry>, Error>;

    /// Returns the diff of a file between two [`Revision`]s.
    async fn get_diff(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        query: &Query,
    ) -> Result<Change, Error>;

    /// Retrieves the diffs of the files matched by the given
    /// path pattern between two [`Revision`]s.
    ///
    /// A path pattern is a variant of glob:
    ///   * `"/**"` - find all files recursively
    ///   * `"*.json"` - find all JSON files recursively
    ///   * `"/foo/*.json"` - find all JSON files under the directory /foo
    ///   * `"/*/foo.txt"` - find all files named foo.txt at the second depth level
    ///   * `"*.json,/bar/*.txt"` - use comma to specify more than one pattern.
    ///   A file will be matched if any pattern matches.
    async fn get_diffs(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        path_pattern: &str,
    ) -> Result<Vec<Change>, Error>;

    /// Retrieves the history of the repository of the files matched by the given
    /// path pattern between two [`Revision`]s.
    /// Note that this method does not retrieve the diffs but only metadata about the changes.
    /// Use [get_diff](#tymethod.get_diff) or
    /// [get_diffs](#tymethod.get_diffs) to retrieve the diffs
    async fn get_history(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        path: &str,
        max_commits: u32,
    ) -> Result<Vec<Commit>, Error>;

    /// Pushes the specified [`Change`]s to the repository.
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
    ) -> Result<Vec<ListEntry>, Error> {
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

/// Watch-related APIs
pub trait WatchService {
    /// Returns a stream which output a [`WatchFileResult`] when the result of the
    /// given [`Query`] becomes available or changes
    fn watch_file_stream(
        &self,
        query: &Query,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchFileResult> + Send>>, Error>;

    /// Returns a stream which output a [`WatchRepoResult`] when the repository has a new commit
    /// that contains the changes for the files matched by the given `path_pattern`.
    fn watch_repo_stream(
        &self,
        path_pattern: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchRepoResult> + Send>>, Error>;
}

impl<'a> WatchService for RepoClient<'a> {
    fn watch_file_stream(
        &self,
        query: &Query,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchFileResult> + Send>>, Error> {
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
    ) -> Result<Pin<Box<dyn Stream<Item = WatchRepoResult> + Send>>, Error> {
        Ok(crate::services::watch::watch_repo_stream(
            self.client.clone(),
            self.project,
            self.repo,
            path_pattern,
        )?
        .boxed())
    }
}
