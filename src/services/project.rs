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

#[cfg(test)]
mod test {
    use super::*;
    use httpmock::{
        Method::{DELETE, GET, PATCH, POST},
        MockServer,
    };

    #[tokio::test]
    async fn test_list_projects() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.path("/api/v1/projects")
                .method(GET)
                .header("Authorization", "Bearer anonymous");
            let resp = r#"[{
                "name":"foo",
                "creator":{"name":"minux", "email":"minux@m.x"},
                "url":"/api/v1/projects/foo"
            }, {
                "name":"bar",
                "creator":{"name":"minux", "email":"minux@m.x"},
                "url":"/api/v1/projects/bar"
            }]"#;
            then.status(200).body(resp.as_bytes());
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        let projects = client.list_projects().await.unwrap();

        mock.assert();
        let expected = [
            ("foo", "minux", "minux@m.x", "/api/v1/projects/foo"),
            ("bar", "minux", "minux@m.x", "/api/v1/projects/bar"),
        ];

        for (p, e) in projects.iter().zip(expected.iter()) {
            assert_eq!(p.name, e.0);
            assert_eq!(p.creator.name, e.1);
            assert_eq!(p.creator.email, e.2);
            assert_eq!(p.url.as_ref().unwrap(), e.3);
        }
    }

    #[tokio::test]
    async fn test_list_removed_projects() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.path("/api/v1/projects")
                .method(GET)
                .query_param("status", "removed")
                .header("Authorization", "Bearer anonymous");
            let resp = r#"[{"name":"foo"}, {"name":"bar"}]"#;
            then.status(200).body(resp.as_bytes());
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        let projects = client.list_removed_projects().await.unwrap();

        mock.assert();
        assert_eq!(projects.len(), 2);

        assert_eq!(projects[0], "foo");
        assert_eq!(projects[1], "bar");
    }

    #[tokio::test]
    async fn test_create_project() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            let project_json = serde_json::json!({"name": "foo"}).to_string();
            when.path("/api/v1/projects")
                .method(POST)
                .body(project_json)
                .header("Authorization", "Bearer anonymous");

            let resp = r#"{
                "name":"foo",
                "creator":{"name":"minux", "email":"minux@m.x"}
            }"#;
            then.status(201).body(resp.as_bytes());
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        let project = client.create_project("foo").await.unwrap();

        mock.assert();

        assert_eq!(project.name, "foo");
        assert_eq!(project.creator.name, "minux");
        assert_eq!(project.creator.email, "minux@m.x");
    }

    #[tokio::test]
    async fn test_remove_project() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.path("/api/v1/projects/foo")
                .method(DELETE)
                .header("Authorization", "Bearer anonymous");
            then.status(204);
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        client.remove_project("foo").await.unwrap();

        mock.assert();
    }

    #[tokio::test]
    async fn test_purge_project() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.path("/api/v1/projects/foo/removed")
                .method(DELETE)
                .header("Authorization", "Bearer anonymous");
            then.status(204);
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        client.purge_project("foo").await.unwrap();

        mock.assert();
    }

    #[tokio::test]
    async fn test_unremove_project() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            let unremove_json = serde_json::json!(
                [{"op": "replace", "path": "/status", "value": "active"}]
            )
            .to_string();
            when.path("/api/v1/projects/foo")
                .method(PATCH)
                .body(unremove_json)
                .header("Content-Type", "application/json-patch+json")
                .header("Authorization", "Bearer anonymous");
            let resp = r#"{
                "name":"foo",
                "creator":{"name":"minux", "email":"minux@m.x"},
                "url":"/api/v1/projects/foo"
            }"#;
            then.status(200).body(resp.as_bytes());
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        let project = client.unremove_project("foo").await.unwrap();

        mock.assert();

        assert_eq!(project.name, "foo");
        assert_eq!(project.creator.name, "minux");
        assert_eq!(project.creator.email, "minux@m.x");
        assert_eq!(project.url.as_ref().unwrap(), "/api/v1/projects/foo");
    }
}
