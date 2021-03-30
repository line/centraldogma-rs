//! Project-related APIs
use crate::{
    client::{Client, Error},
    model::Project,
    services::{path, status_unwrap},
};

use async_trait::async_trait;
use reqwest::{Body, Method};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Project-related APIs
#[async_trait]
pub trait ProjectService {
    /// Creates a project.
    async fn create_project(&self, name: &str) -> Result<Project, Error>;

    /// Removes a project. A removed project can be [unremoved](#tymethod.unremove_project).
    async fn remove_project(&self, name: &str) -> Result<(), Error>;

    /// Purges a project that was removed before.
    async fn purge_project(&self, name: &str) -> Result<(), Error>;

    /// Unremoves a project.
    async fn unremove_project(&self, name: &str) -> Result<Project, Error>;

    /// Retrieves the list of the projects.
    async fn list_projects(&self) -> Result<Vec<Project>, Error>;

    /// Retrieves the list of the removed projects,
    /// which can be [unremoved](#tymethod.unremove_project)
    /// or [purged](#tymethod.purge_project).
    async fn list_removed_projects(&self) -> Result<Vec<String>, Error>;
}

#[async_trait]
impl ProjectService for Client {
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

    async fn purge_project(&self, name: &str) -> Result<(), Error> {
        let req = self.new_request(Method::DELETE, path::removed_project_path(name), None)?;

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
}

#[cfg(test)]
mod test {
    use super::*;
    use wiremock::{
        matchers::{body_json, header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn test_list_projects() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200).set_body_raw(
            r#"[{
                "name":"foo",
                "creator":{"name":"minux", "email":"minux@m.x"},
                "url":"/api/v1/projects/foo"
            }, {
                "name":"bar",
                "creator":{"name":"minux", "email":"minux@m.x"},
                "url":"/api/v1/projects/bar"
            }]"#,
            "application/json",
        );
        Mock::given(method("GET"))
            .and(path("/api/v1/projects"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let projects = client.list_projects().await.unwrap();

        drop(server);
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
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200).set_body_raw(
            r#"[
                {"name":"foo"},
                {"name":"bar"}
            ]"#,
            "application/json",
        );
        Mock::given(method("GET"))
            .and(path("/api/v1/projects"))
            .and(query_param("status", "removed"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let projects = client.list_removed_projects().await.unwrap();

        drop(server);
        assert_eq!(projects.len(), 2);

        assert_eq!(projects[0], "foo");
        assert_eq!(projects[1], "bar");
    }

    #[tokio::test]
    async fn test_create_project() {
        let server = MockServer::start().await;
        let project_json = serde_json::json!({"name": "foo"});
        let resp = ResponseTemplate::new(201).set_body_raw(
            r#"{
                "name":"foo",
                "creator":{"name":"minux", "email":"minux@m.x"}
            }"#,
            "application/json",
        );
        Mock::given(method("POST"))
            .and(path("/api/v1/projects"))
            .and(header("Authorization", "Bearer anonymous"))
            .and(body_json(project_json))
            .respond_with(resp)
            .expect(1)
            .mount(&server)
            .await;
        let client = Client::new(&server.uri(), None).await.unwrap();
        let project = client.create_project("foo").await.unwrap();

        drop(server);

        assert_eq!(project.name, "foo");
        assert_eq!(project.creator.name, "minux");
        assert_eq!(project.creator.email, "minux@m.x");
    }

    #[tokio::test]
    async fn test_remove_project() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(204);
        Mock::given(method("DELETE"))
            .and(path("/api/v1/projects/foo"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        client.remove_project("foo").await.unwrap();
    }

    #[tokio::test]
    async fn test_purge_project() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(204);
        Mock::given(method("DELETE"))
            .and(path("/api/v1/projects/foo/removed"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        client.purge_project("foo").await.unwrap();
    }

    #[tokio::test]
    async fn test_unremove_project() {
        let server = MockServer::start().await;
        let unremove_json =
            serde_json::json!([{"op": "replace", "path": "/status", "value": "active"}]);
        let resp = ResponseTemplate::new(201).set_body_raw(
            r#"{
                "name":"foo",
                "creator":{"name":"minux", "email":"minux@m.x"},
                "url":"/api/v1/projects/foo"
            }"#,
            "application/json",
        );
        Mock::given(method("PATCH"))
            .and(path("/api/v1/projects/foo"))
            .and(header("Content-Type", "application/json-patch+json"))
            .and(header("Authorization", "Bearer anonymous"))
            .and(body_json(unremove_json))
            .respond_with(resp)
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let project = client.unremove_project("foo").await.unwrap();

        drop(server);

        assert_eq!(project.name, "foo");
        assert_eq!(project.creator.name, "minux");
        assert_eq!(project.creator.email, "minux@m.x");
        assert_eq!(project.url.as_ref().unwrap(), "/api/v1/projects/foo");
    }
}
