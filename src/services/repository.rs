//! Repository-related APIs
use crate::{
    client::{status_unwrap, Error, ProjectClient},
    model::Repository,
    path,
};

use async_trait::async_trait;
use reqwest::{Body, Method};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Repository-related APIs
#[async_trait]
pub trait RepoService {
    /// Retrieves the list of the repositories.
    async fn list_repos(&self) -> Result<Vec<Repository>, Error>;

    /// Retrieves the list of the removed repositories, which can be
    /// [unremoved](#tymethod.unremove_repo).
    async fn list_removed_repos(&self) -> Result<Vec<String>, Error>;

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

    async fn list_removed_repos(&self) -> Result<Vec<String>, Error> {
        #[derive(Deserialize)]
        struct RemovedRepo {
            name: String,
        }
        let req =
            self.client
                .new_request(Method::GET, path::removed_repos_path(self.project), None)?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        if ok_resp.status().as_u16() == 204 {
            return Ok(Vec::new());
        }
        let result: Vec<RemovedRepo> = ok_resp.json().await?;
        let result = result.into_iter().map(|r| r.name).collect();

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        model::{Author, Revision},
        Client,
    };
    use httpmock::{
        Method::{DELETE, GET, PATCH, POST},
        MockServer,
    };

    #[tokio::test]
    async fn test_list_repos() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.path("/api/v1/projects/foo/repos")
                .method(GET)
                .header("Authorization", "Bearer anonymous");
            let resp = r#"[{
                "name":"bar",
                "creator":{"name":"minux", "email":"minux@m.x"},
                "url":"/api/v1/projects/foo/repos/bar",
                "createdAt":"a",
                "headRevision":2
            },{
                "name":"baz",
                "creator":{"name":"minux", "email":"minux@m.x"},
                "url":"/api/v1/projects/foo/repos/baz",
                "createdAt":"a",
                "headRevision":3
            }]"#;
            then.status(200).body(resp.as_bytes());
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        let repos = client.project("foo").list_repos().await.unwrap();

        mock.assert();
        let expected = [
            (
                "bar",
                Author {
                    name: "minux".to_string(),
                    email: "minux@m.x".to_string(),
                },
                "/api/v1/projects/foo/repos/bar",
                Revision::from(2),
            ),
            (
                "baz",
                Author {
                    name: "minux".to_string(),
                    email: "minux@m.x".to_string(),
                },
                "/api/v1/projects/foo/repos/baz",
                Revision::from(3),
            ),
        ];

        for (r, e) in repos.iter().zip(expected.iter()) {
            assert_eq!(r.name, e.0);
            assert_eq!(r.creator, e.1);
            assert_eq!(r.url.as_ref().unwrap(), &e.2);
            assert_eq!(r.head_revision, e.3);
        }
    }

    #[tokio::test]
    async fn test_list_removed_repos() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.path("/api/v1/projects/foo/repos")
                .method(GET)
                .query_param("status", "removed")
                .header("Authorization", "Bearer anonymous");
            let resp = r#"[{"name":"bar"}, {"name":"baz"}]"#;
            then.status(200).body(resp.as_bytes());
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        let repos = client.project("foo").list_removed_repos().await.unwrap();

        mock.assert();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0], "bar");
        assert_eq!(repos[1], "baz");
    }

    #[tokio::test]
    async fn test_create_repos() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            let repo_json = serde_json::json!({"name": "bar"});
            when.path("/api/v1/projects/foo/repos")
                .method(POST)
                .json_body(repo_json)
                .header("Authorization", "Bearer anonymous");

            let resp = r#"{"name":"bar",
                "creator":{"name":"minux", "email":"minux@m.x"},
                "createdAt":"a",
                "headRevision": 2}"#;
            then.status(201).body(resp.as_bytes());
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        let repo = client.project("foo").create_repo("bar").await.unwrap();

        mock.assert();
        assert_eq!(repo.name, "bar");
        assert_eq!(
            repo.creator,
            Author {
                name: "minux".to_string(),
                email: "minux@m.x".to_string()
            }
        );
        assert_eq!(repo.head_revision, Revision::from(2));
    }

    #[tokio::test]
    async fn test_remove_repos() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.path("/api/v1/projects/foo/repos/bar")
                .method(DELETE)
                .header("Authorization", "Bearer anonymous");
            then.status(204);
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        client.project("foo").remove_repo("bar").await.unwrap();

        mock.assert();
    }

    #[tokio::test]
    async fn test_purge_repos() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.path("/api/v1/projects/foo/repos/bar/removed")
                .method(DELETE)
                .header("Authorization", "Bearer anonymous");
            then.status(204);
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        client.project("foo").purge_repo("bar").await.unwrap();

        mock.assert();
    }

    #[tokio::test]
    async fn test_unremove_repos() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            let unremove_json = serde_json::json!(
                [{"op": "replace", "path": "/status", "value": "active"}]
            )
            .to_string();
            when.path("/api/v1/projects/foo/repos/bar")
                .method(PATCH)
                .header("Content-Type", "application/json-patch+json")
                .body(unremove_json)
                .header("Authorization", "Bearer anonymous");

            let resp = r#"{"name":"bar",
                "creator":{"name":"minux", "email":"minux@m.x"},
                "createdAt":"a",
                "url":"/api/v1/projects/foo/repos/bar",
                "headRevision": 2}"#;
            then.status(200).body(resp.as_bytes());
        });

        let client = Client::new(&server.base_url(), None).await.unwrap();
        let repo = client.project("foo").unremove_repo("bar").await;

        mock.assert_async().await;
        let repo = repo.unwrap();
        assert_eq!(repo.name, "bar");
        assert_eq!(
            repo.creator,
            Author {
                name: "minux".to_string(),
                email: "minux@m.x".to_string()
            }
        );
        assert_eq!(repo.head_revision, Revision::from(2));
    }
}
