use crate::{
    client::{self, status_unwrap, Client},
    model::Repository,
    services::consts,
};

use reqwest::{Body, Method};
use serde::Serialize;
use serde_json::json;

pub async fn list_by_project_name(
    client: &Client,
    project_name: &str,
) -> Result<Vec<Repository>, client::Error> {
    let req = client.new_request(
        Method::GET,
        format!(
            "{}/{}/{}/{}",
            consts::PATH_PREFIX,
            consts::PROJECT_PATH,
            project_name,
            consts::REPO_PATH
        ),
        None,
    )?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

pub async fn list_removed_by_project_name(
    client: &Client,
    project_name: &str,
) -> Result<Vec<Repository>, client::Error> {
    let req = client.new_request(
        Method::GET,
        format!(
            "{}/{}/{}/{}?status=removed",
            consts::PATH_PREFIX,
            consts::PROJECT_PATH,
            project_name,
            consts::REPO_PATH
        ),
        None,
    )?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    if ok_resp.status().as_u16() == 204 {
        return Ok(Vec::new());
    }
    let result = ok_resp.json().await?;

    Ok(result)
}

pub async fn create(
    client: &Client,
    project_name: &str,
    repo_name: &str
) -> Result<Repository, client::Error> {
    #[derive(Serialize)]
    struct CreateRepo<'a> {
        name: &'a str,
    };
    let body = serde_json::to_vec(&CreateRepo { name: repo_name })?;
    let body = Body::from(body);

    let req = client.new_request(
        Method::POST,
        format!(
            "{}/{}/{}/{}",
            consts::PATH_PREFIX,
            consts::PROJECT_PATH,
            project_name,
            consts::REPO_PATH
        ),
        Some(body),
    )?;

    let resp = client.request(req).await?;
    let resp_body = status_unwrap(resp).await?
        .bytes().await?;
    let result = serde_json::from_slice(&resp_body[..])?;

    Ok(result)
}

pub async fn remove(
    client: &Client,
    project_name: &str,
    repo_name: &str
) -> Result<(), client::Error> {
    let req = client.new_request(
        Method::DELETE,
        format!(
            "{}/{}/{}/{}/{}",
            consts::PATH_PREFIX,
            consts::PROJECT_PATH,
            project_name,
            consts::REPO_PATH,
            repo_name
        ),
        None
    )?;

    let resp = client.request(req).await?;
    let _ = status_unwrap(resp).await?;

    Ok(())
}

pub async fn unremove(
    client: &Client,
    project_name: &str,
    repo_name: &str
) -> Result<Repository, client::Error> {
    let body: Vec<u8> = serde_json::to_vec(&json!([
        {"op":"replace", "path":"/status", "value":"active"}
    ]))?;
    let body = Body::from(body);
    let req = client.new_request(
        Method::PATCH,
        format!(
            "{}/{}/{}/{}/{}",
            consts::PATH_PREFIX,
            consts::PROJECT_PATH,
            project_name,
            consts::REPO_PATH,
            repo_name
        ),
        Some(body)
    )?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

pub async fn purge(
    client: &Client,
    project_name: &str,
    repo_name: &str
) -> Result<(), client::Error> {
    let req = client.new_request(
        Method::DELETE,
        format!(
            "{}/{}/{}/{}/{}/removed",
            consts::PATH_PREFIX,
            consts::PROJECT_PATH,
            project_name,
            consts::REPO_PATH,
            repo_name
        ),
        None
    )?;

    let resp = client.request(req).await?;
    let _ = status_unwrap(resp).await?;

    Ok(())
}
