//! Content-related APIs
use crate::{
    client::status_unwrap,
    model::{Change, Commit, CommitMessage, Entry, ListEntry, PushResult, Query, Revision},
    path, Client, Error,
};

use reqwest::{Body, Method};
use serde::Serialize;

/// Retrieves the list of the files at the specified [`Revision`] matched by the path pattern.
///
/// A path pattern is a variant of glob:
///   * `"/**"` - find all files recursively
///   * `"*.json"` - find all JSON files recursively
///   * `"/foo/*.json"` - find all JSON files under the directory /foo
///   * `"/*/foo.txt"` - find all files named foo.txt at the second depth level
///   * `"*.json,/bar/*.txt"` - use comma to specify more than one pattern.
///   A file will be matched if any pattern matches.
pub async fn list_files(
    client: &Client,
    project_name: &str,
    repo_name: &str,
    revision: Revision,
    path_pattern: &str,
) -> Result<Vec<ListEntry>, Error> {
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

/// Queries a file at the specified [`Revision`] and path with the specified [`Query`].
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

/// Retrieves the files at the specified [`Revision`] matched by the path pattern.
///
/// A path pattern is a variant of glob:
///   * `"/**"` - find all files recursively
///   * `"*.json"` - find all JSON files recursively
///   * `"/foo/*.json"` - find all JSON files under the directory /foo
///   * `"/*/foo.txt"` - find all files named foo.txt at the second depth level
///   * `"*.json,/bar/*.txt"` - use comma to specify more than one pattern.
///   A file will be matched if any pattern matches.
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

/// Retrieves the history of the repository of the files matched by the given
/// path pattern between two [`Revision`]s.
/// Note that this method does not retrieve the diffs but only metadata about the changes.
/// Use [get_diff](#tymethod.get_diff) or
/// [get_diffs](#tymethod.get_diffs) to retrieve the diffs
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

/// Returns the diff of a file between two [`Revision`]s.
pub async fn get_diff(
    client: &Client,
    project_name: &str,
    repo_name: &str,
    from_rev: Revision,
    to_rev: Revision,
    query: &Query,
) -> Result<Change, Error> {
    let p = path::content_compare_path(project_name, repo_name, from_rev, to_rev, query);
    let req = client.new_request(Method::GET, p, None)?;

    let resp = client.request(req).await?;
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(result)
}

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

/// Pushes the specified [`Change`]s to the repository.
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
