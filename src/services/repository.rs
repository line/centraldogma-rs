//! Repository-related APIs
use crate::{
    client::{status_unwrap, Error, ProjectClient},
    model::Repository,
    path,
};

use async_trait::async_trait;
use reqwest::{Body, Method};
use serde::Serialize;
use serde_json::json;

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

#[async_trait]
impl<'a> RepoService for ProjectClient<'a> {
    async fn list_repos(&self) -> Result<Vec<Repository>, Error> {
        let req = self
            .client
            .new_request(Method::GET, path::repos_path(self.project), None)?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn list_removed_repos(&self) -> Result<Vec<Repository>, Error> {
        let req =
            self.client
                .new_request(Method::GET, path::removed_repos_path(self.project), None)?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        if ok_resp.status().as_u16() == 204 {
            return Ok(Vec::new());
        }
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn create_repo(&self, repo_name: &str) -> Result<Repository, Error> {
        #[derive(Serialize)]
        struct CreateRepo<'a> {
            name: &'a str,
        }

        let body = serde_json::to_vec(&CreateRepo { name: repo_name })?;
        let body = Body::from(body);

        let req =
            self.client
                .new_request(Method::POST, path::repos_path(self.project), Some(body))?;

        let resp = self.client.request(req).await?;
        let resp_body = status_unwrap(resp).await?.bytes().await?;
        let result = serde_json::from_slice(&resp_body[..])?;

        Ok(result)
    }

    async fn remove_repo(&self, repo_name: &str) -> Result<(), Error> {
        let req = self.client.new_request(
            Method::DELETE,
            path::repo_path(self.project, repo_name),
            None,
        )?;

        let resp = self.client.request(req).await?;
        let _ = status_unwrap(resp).await?;

        Ok(())
    }

    async fn unremove_repo(&self, repo_name: &str) -> Result<Repository, Error> {
        let body: Vec<u8> = serde_json::to_vec(&json!([
            {"op":"replace", "path":"/status", "value":"active"}
        ]))?;
        let body = Body::from(body);
        let req = self.client.new_request(
            Method::PATCH,
            path::repo_path(self.project, repo_name),
            Some(body),
        )?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn purge_repo(&self, repo_name: &str) -> Result<(), Error> {
        let req = self.client.new_request(
            Method::DELETE,
            path::removed_repo_path(self.project, repo_name),
            None,
        )?;

        let resp = self.client.request(req).await?;
        let _ = status_unwrap(resp).await?;

        Ok(())
    }
}
