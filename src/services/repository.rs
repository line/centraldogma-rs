//! Repository-related APIs
use crate::{
    client::{self, status_unwrap, Client},
    model::Repository,
    path,
};

use reqwest::{Body, Method};
use serde::Serialize;
use serde_json::json;

/// Retrieves the list of the repositories from the specified project.
pub async fn list_by_project_name(
    client: &Client,
    project_name: &str,
) -> Result<Vec<Repository>, client::Error> {
    let req = client.new_request(Method::GET, path::repos_path(project_name), None)?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

/// Retrieves the list of the removed repositories from the specified project ,
/// which can be [unremoved](unremove).
pub async fn list_removed_by_project_name(
    client: &Client,
    project_name: &str,
) -> Result<Vec<Repository>, client::Error> {
    let req = client.new_request(Method::GET, path::removed_repos_path(project_name), None)?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    if ok_resp.status().as_u16() == 204 {
        return Ok(Vec::new());
    }
    let result = ok_resp.json().await?;

    Ok(result)
}

/// Creates a repository in the specified project.
pub async fn create(
    client: &Client,
    project_name: &str,
    repo_name: &str,
) -> Result<Repository, client::Error> {
    #[derive(Serialize)]
    struct CreateRepo<'a> {
        name: &'a str,
    }

    let body = serde_json::to_vec(&CreateRepo { name: repo_name })?;
    let body = Body::from(body);

    let req = client.new_request(Method::POST, path::repos_path(project_name), Some(body))?;

    let resp = client.request(req).await?;
    let resp_body = status_unwrap(resp).await?.bytes().await?;
    let result = serde_json::from_slice(&resp_body[..])?;

    Ok(result)
}

/// Removes a repository, removed repository can be
/// [unremoved](unremove).
pub async fn remove(
    client: &Client,
    project_name: &str,
    repo_name: &str,
) -> Result<(), client::Error> {
    let req = client.new_request(
        Method::DELETE,
        path::repo_path(project_name, repo_name),
        None,
    )?;

    let resp = client.request(req).await?;
    let _ = status_unwrap(resp).await?;

    Ok(())
}

/// Unremoves a repository.
pub async fn unremove(
    client: &Client,
    project_name: &str,
    repo_name: &str,
) -> Result<Repository, client::Error> {
    let body: Vec<u8> = serde_json::to_vec(&json!([
        {"op":"replace", "path":"/status", "value":"active"}
    ]))?;
    let body = Body::from(body);
    let req = client.new_request(
        Method::PATCH,
        path::repo_path(project_name, repo_name),
        Some(body),
    )?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

/// Purges a repository that was removed before.
pub async fn purge(
    client: &Client,
    project_name: &str,
    repo_name: &str,
) -> Result<(), client::Error> {
    let req = client.new_request(
        Method::DELETE,
        path::removed_repo_path(project_name, repo_name),
        None,
    )?;

    let resp = client.request(req).await?;
    let _ = status_unwrap(resp).await?;

    Ok(())
}
