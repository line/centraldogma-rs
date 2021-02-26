use crate::{
    client::status_unwrap,
    model::{Change, Commit, CommitMessage, PushResult, Query, Revision},
    path, Client, Entry, Error,
};

use reqwest::{Body, Method};
use serde::Serialize;

pub async fn list_files(
    client: &Client,
    project_name: &str,
    repo_name: &str,
    revision: Revision,
    path_pattern: &str,
) -> Result<Vec<Entry>, Error> {
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
    revision: Revision,
    query: &Query,
) -> Result<Entry, Error> {
    let p = path::content_path(project_name, repo_name, revision, query);
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
    revision: Revision,
    path_pattern: &str,
) -> Result<Vec<Entry>, Error> {
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
    from_rev: Revision,
    to_rev: Revision,
    path: &str,
    max_commits: u32,
) -> Result<Vec<Commit>, Error> {
    let p =
        path::content_commits_path(project_name, repo_name, from_rev, to_rev, path, max_commits);
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
    from_rev: Revision,
    to_rev: Revision,
    query: &Query,
) -> Result<Change, Error> {
    if query.path.is_empty() {
        return Err(Error::InvalidParams(
            "get_diff query path should not be empty",
        ));
    }
    let p = path::content_compare_path(project_name, repo_name, from_rev, to_rev, query);
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
    from_rev: Revision,
    to_rev: Revision,
    path_pattern: &str,
) -> Result<Vec<Change>, Error> {
    let p = path::contents_compare_path(project_name, repo_name, from_rev, to_rev, path_pattern);
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
    base_revision: Revision,
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
