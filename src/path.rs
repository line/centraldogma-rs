use std::borrow::Cow;

use crate::model::{Query, QueryType};

const PATH_PREFIX: &str = "/api/v1";

pub(crate) fn projects_path() -> String {
    format!("{}/projects", PATH_PREFIX)
}

pub(crate) fn removed_projects_path() -> String {
    format!("{}/projects?status=removed", PATH_PREFIX)
}

pub(crate) fn project_path(project_name: &str) -> String {
    format!("{}/projects/{}", PATH_PREFIX, project_name)
}

pub(crate) fn removed_project_path(project_name: &str) -> String {
    format!("{}/projects/{}/removed", PATH_PREFIX, project_name)
}

pub(crate) fn repos_path(project_name: &str) -> String {
    format!("{}/projects/{}/repos", PATH_PREFIX, project_name)
}

pub(crate) fn removed_repos_path(project_name: &str) -> String {
    format!(
        "{}/projects/{}/repos?status=removed",
        PATH_PREFIX, project_name
    )
}

pub(crate) fn repo_path(project_name: &str, repo_name: &str) -> String {
    format!(
        "{}/projects/{}/repos/{}",
        PATH_PREFIX, project_name, repo_name
    )
}

pub(crate) fn removed_repo_path(project_name: &str, repo_name: &str) -> String {
    format!(
        "{}/projects/{}/repos/{}/removed",
        PATH_PREFIX, project_name, repo_name
    )
}

pub(crate) fn list_contents_path(
    project_name: &str,
    repo_name: &str,
    revision: i64,
    path_pattern: &str,
) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/list/{}?",
        PATH_PREFIX, project_name, repo_name, path_pattern
    );
    let len = url.len();

    form_urlencoded::Serializer::for_suffix(url, len)
        .append_pair("revision", &revision.to_string())
        .finish()
}

pub(crate) fn contents_path(
    project_name: &str,
    repo_name: &str,
    revision: i64,
    path_pattern: &str,
) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/contents/{}?",
        PATH_PREFIX, project_name, repo_name, path_pattern
    );
    let len = url.len();

    form_urlencoded::Serializer::for_suffix(url, len)
        .append_pair("revision", &revision.to_string())
        .finish()
}

pub(crate) fn content_path(
    project_name: &str,
    repo_name: &str,
    revision: i64,
    query: &Query,
) -> Option<String> {
    let path = if query.path.starts_with("/") {
        &query.path[1..]
    } else {
        &query.path
    };

    let url = format!(
        "{}/projects/{}/repos/{}/contents/{}?",
        PATH_PREFIX, project_name, repo_name, path
    );

    let len = url.len();
    let mut serializer = form_urlencoded::Serializer::for_suffix(url, len);
    serializer.append_pair("revision", &revision.to_string());

    if let QueryType::JsonPath(expressions) = &query.r#type {
        if !query.path.to_lowercase().ends_with("json") {
            return None;
        }

        for expression in expressions.iter() {
            serializer.append_pair("jsonpath", expression);
        }
    }

    Some(serializer.finish())
}

pub(crate) fn content_commits_path(
    project_name: &str,
    repo_name: &str,
    from: &str,
    to: &str,
    path: &str,
    max_commits: u32,
) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/commits/{}?",
        PATH_PREFIX, project_name, repo_name, from
    );

    let len = url.len();
    form_urlencoded::Serializer::for_suffix(url, len)
        .append_pair("path", path)
        .append_pair("to", to)
        .append_pair("maxCommits", &max_commits.to_string())
        .finish()
}

pub(crate) fn content_compare_path(
    project_name: &str,
    repo_name: &str,
    from: &str,
    to: &str,
    query: &Query,
) -> Option<String> {
    let url = format!(
        "{}/projects/{}/repos/{}/compare?",
        PATH_PREFIX, project_name, repo_name
    );

    let path = if query.path.starts_with("/") {
        Cow::Borrowed(&query.path)
    } else {
        Cow::Owned(format!("/{}", query.path))
    };

    let len = url.len();
    let mut serializer = form_urlencoded::Serializer::for_suffix(url, len);
    serializer
        .append_pair("path", &path)
        .append_pair("from", from)
        .append_pair("to", to);

    if let QueryType::JsonPath(expressions) = &query.r#type {
        if !query.path.to_lowercase().ends_with("json") {
            return None;
        }

        for expression in expressions.iter() {
            serializer.append_pair("jsonpath", expression);
        }
    }

    Some(serializer.finish())
}

pub(crate) fn contents_compare_path(
    project_name: &str,
    repo_name: &str,
    from: &str,
    to: &str,
    path_pattern: &str,
) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/compare?",
        PATH_PREFIX, project_name, repo_name
    );

    let len = url.len();
    form_urlencoded::Serializer::for_suffix(url, len)
        .append_pair("pathPattern", path_pattern)
        .append_pair("from", from)
        .append_pair("to", to)
        .finish()
}

pub(crate) fn contents_push_path(
    project_name: &str,
    repo_name: &str,
    base_revision: i64,
) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/contents?",
        PATH_PREFIX, project_name, repo_name
    );

    let len = url.len();
    form_urlencoded::Serializer::for_suffix(url, len)
        .append_pair("revision", &base_revision.to_string())
        .finish()
}
