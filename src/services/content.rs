use std::borrow::Cow;

use crate::{
    client::status_unwrap,
    model::{Change, Commit, CommitMessage, PushResult, Query},
    path, Client, Entry, Error,
};

use reqwest::{Body, Method};
use serde::Serialize;

fn normalize_path_pattern(path_pattern: &str) -> Cow<str> {
    if path_pattern.is_empty() {
        return Cow::Borrowed("/**");
    }
    if path_pattern.starts_with("**") {
        return Cow::Owned(format!("/{}", path_pattern));
    }
    if path_pattern.starts_with("/") {
        return Cow::Owned(format!("/**/{}", path_pattern));
    }

    Cow::Borrowed(path_pattern)
}

pub async fn list_files(
    client: &Client,
    project_name: &str,
    repo_name: &str,
    revision: i64,
    path_pattern: &str,
) -> Result<Vec<Entry>, Error> {
    let path_pattern = normalize_path_pattern(path_pattern);
    let req = client.new_request(
        Method::GET,
        path::list_contents_path(project_name, repo_name, revision, &path_pattern),
        None,
    )?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

pub async fn get_file(
    client: &Client,
    project_name: &str,
    repo_name: &str,
    revision: i64,
    query: &Query,
) -> Result<Entry, Error> {
    let p = path::content_path(project_name, repo_name, revision, query)
        .ok_or_else(|| Error::InvalidParams("JsonPath type only applicable to .json file"))?;
    let req = client.new_request(Method::GET, p, None)?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

pub async fn get_files(
    client: &Client,
    project_name: &str,
    repo_name: &str,
    revision: i64,
    path_pattern: &str,
) -> Result<Vec<Entry>, Error> {
    let path_pattern = normalize_path_pattern(path_pattern);
    let req = client.new_request(
        Method::GET,
        path::contents_path(project_name, repo_name, revision, &path_pattern),
        None,
    )?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

pub async fn get_history(
    client: &Client,
    project_name: &str,
    repo_name: &str,
    from: &str,
    to: &str,
    path: &str,
    max_commits: u32,
) -> Result<Vec<Commit>, Error> {
    let p = path::content_commits_path(project_name, repo_name, from, to, path, max_commits);
    let req = client.new_request(Method::GET, p, None)?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

pub async fn get_diff(
    client: &Client,
    project_name: &str,
    repo_name: &str,
    from: &str,
    to: &str,
    query: &Query,
) -> Result<Change, Error> {
    if query.path == "" {
        return Err(Error::InvalidParams(
            "get_diff query path should not be empty",
        ));
    }
    let p = path::content_compare_path(project_name, repo_name, from, to, query)
        .ok_or_else(|| Error::InvalidParams("JsonPath type only applicable to .json file"))?;
    let req = client.new_request(Method::GET, p, None)?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

pub async fn get_diffs(
    client: &Client,
    project_name: &str,
    repo_name: &str,
    from: &str,
    to: &str,
    path_pattern: &str,
) -> Result<Vec<Change>, Error> {
    let path_pattern = if path_pattern.is_empty() {
        "/**"
    } else {
        path_pattern
    };

    let p = path::contents_compare_path(project_name, repo_name, from, to, path_pattern);
    let req = client.new_request(Method::GET, p, None)?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Push {
    commit_message: CommitMessage,
    changes: Vec<Change>,
}

pub async fn push(
    client: &Client,
    project_name: &str,
    repo_name: &str,
    base_revision: i64,
    commit_message: CommitMessage,
    changes: Vec<Change>,
) -> Result<PushResult, Error> {
    if commit_message.summary.is_empty() {
        return Err(Error::InvalidParams(
            "summary of commit_message cannot be empty",
        ));
    }
    if changes.is_empty() {
        return Err(Error::InvalidParams("no changes to commit"));
    }

    let body: String = serde_json::to_string(&Push {
        commit_message,
        changes,
    })?;
    let body = Body::from(body);

    let p = path::contents_push_path(project_name, repo_name, base_revision);
    let req = client.new_request(Method::POST, p, Some(body))?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}
