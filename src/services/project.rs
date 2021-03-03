//! Project-related APIs
use crate::{
    client::{status_unwrap, Client, Error},
    model::Project,
    path,
};

use async_trait::async_trait;
use reqwest::{Body, Method};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
    async fn create_project(&self, name: &str) -> Result<Project, Error>;

    /// Removes a project. A removed project can be [unremoved](#tymethod.unremove_project).
    async fn remove_project(&self, name: &str) -> Result<(), Error>;

    /// Unremoves a project.
    async fn unremove_project(&self, name: &str) -> Result<Project, Error>;

    /// Purges a project that was removed before.
    async fn purge_project(&self, name: &str) -> Result<(), Error>;
}

#[async_trait]
impl ProjectService for Client {
    async fn list_projects(&self) -> Result<Vec<Project>, Error> {
        let req = self.new_request(Method::GET, path::projects_path(), None)?;
        let resp = self.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;

        if let Some(0) = ok_resp.content_length() {
            return Ok(Vec::new());
        }
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn list_removed_projects(&self) -> Result<Vec<String>, Error> {
        #[derive(Deserialize)]
        struct RemovedProject {
            name: String,
        }
        let req = self.new_request(Method::GET, path::removed_projects_path(), None)?;
        let resp = self.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;

        let result: Vec<RemovedProject> = ok_resp.json().await?;
        let result = result.into_iter().map(|p| p.name).collect();

        Ok(result)
    }

    async fn create_project(&self, name: &str) -> Result<Project, Error> {
        #[derive(Serialize)]
        struct CreateProject<'a> {
            name: &'a str,
        }

        let body: Vec<u8> = serde_json::to_vec(&CreateProject { name })?;
        let body = Body::from(body);
        let req = self.new_request(Method::POST, path::projects_path(), Some(body))?;

        let resp = self.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn remove_project(&self, name: &str) -> Result<(), Error> {
        let req = self.new_request(Method::DELETE, path::project_path(name), None)?;

        let resp = self.request(req).await?;
        let _ = status_unwrap(resp).await?;

        Ok(())
    }

    async fn unremove_project(&self, name: &str) -> Result<Project, Error> {
        let body: Vec<u8> = serde_json::to_vec(&json!([
            {"op":"replace", "path":"/status", "value":"active"}
        ]))?;
        let body = Body::from(body);
        let req = self.new_request(Method::PATCH, path::project_path(name), Some(body))?;

        let resp = self.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn purge_project(&self, name: &str) -> Result<(), Error> {
        let req = self.new_request(Method::DELETE, path::removed_project_path(name), None)?;

        let resp = self.request(req).await?;
        let _ = status_unwrap(resp).await?;

        Ok(())
    }
}
