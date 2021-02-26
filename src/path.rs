use std::borrow::Cow;

use crate::model::{Query, QueryType, Revision};

const PATH_PREFIX: &str = "/api/v1";

fn normalize_path_pattern(path_pattern: &str) -> Cow<str> {
    if path_pattern.is_empty() {
        return Cow::Borrowed("/**");
    }
    if path_pattern.starts_with("**") {
        return Cow::Owned(format!("/{}", path_pattern));
    }
    if !path_pattern.starts_with('/') {
        return Cow::Owned(format!("/**/{}", path_pattern));
    }

    Cow::Borrowed(path_pattern)
}

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
    revision: Revision,
    path_pattern: &str,
) -> String {
    let path_pattern = normalize_path_pattern(path_pattern);
    let url = format!(
        "{}/projects/{}/repos/{}/list{}?",
        PATH_PREFIX, project_name, repo_name, &path_pattern
    );
    let len = url.len();

    form_urlencoded::Serializer::for_suffix(url, len)
        .append_pair("revision", &revision.to_string())
        .finish()
}

pub(crate) fn contents_path(
    project_name: &str,
    repo_name: &str,
    revision: Revision,
    path_pattern: &str,
) -> String {
    let path_pattern = normalize_path_pattern(path_pattern);
    let url = format!(
        "{}/projects/{}/repos/{}/contents{}?",
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
    revision: Revision,
    query: &Query,
) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/contents{}?",
        PATH_PREFIX, project_name, repo_name, &query.path
    );

    let len = url.len();
    let mut serializer = form_urlencoded::Serializer::for_suffix(url, len);
    serializer.append_pair("revision", &revision.to_string());

    if let QueryType::JsonPath(expressions) = &query.r#type {
        for expression in expressions.iter() {
            serializer.append_pair("jsonpath", expression);
        }
    }

    serializer.finish()
}

pub(crate) fn content_commits_path(
    project_name: &str,
    repo_name: &str,
    from_rev: Revision,
    to_rev: Revision,
    path: &str,
    max_commits: u32,
) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/commits/{}?",
        PATH_PREFIX,
        project_name,
        repo_name,
        &from_rev.to_string(),
    );

    let len = url.len();
    form_urlencoded::Serializer::for_suffix(url, len)
        .append_pair("path", path)
        .append_pair("to", &to_rev.to_string())
        .append_pair("maxCommits", &max_commits.to_string())
        .finish()
}

pub(crate) fn content_compare_path(
    project_name: &str,
    repo_name: &str,
    from_rev: Revision,
    to_rev: Revision,
    query: &Query,
) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/compare?",
        PATH_PREFIX, project_name, repo_name
    );

    let len = url.len();
    let mut serializer = form_urlencoded::Serializer::for_suffix(url, len);
    serializer
        .append_pair("path", &query.path)
        .append_pair("from", &from_rev.to_string())
        .append_pair("to", &to_rev.to_string());

    if let QueryType::JsonPath(expressions) = &query.r#type {
        for expression in expressions.iter() {
            serializer.append_pair("jsonpath", expression);
        }
    }

    serializer.finish()
}

pub(crate) fn contents_compare_path(
    project_name: &str,
    repo_name: &str,
    from_rev: Revision,
    to_rev: Revision,
    path_pattern: &str,
) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/compare?",
        PATH_PREFIX, project_name, repo_name
    );

    let path_pattern = normalize_path_pattern(path_pattern);
    let len = url.len();
    form_urlencoded::Serializer::for_suffix(url, len)
        .append_pair("pathPattern", &path_pattern)
        .append_pair("from", &from_rev.to_string())
        .append_pair("to", &to_rev.to_string())
        .finish()
}

pub(crate) fn contents_push_path(
    project_name: &str,
    repo_name: &str,
    base_revision: Revision,
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

pub(crate) fn content_watch_path(project_name: &str, repo_name: &str, query: &Query) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/contents{}?",
        PATH_PREFIX, project_name, repo_name, &query.path
    );

    let len = url.len();
    let mut serializer = form_urlencoded::Serializer::for_suffix(url, len);

    if let QueryType::JsonPath(expressions) = &query.r#type {
        for expression in expressions.iter() {
            serializer.append_pair("jsonpath", expression);
        }
    }

    serializer.finish()
}

pub(crate) fn repo_watch_path(project_name: &str, repo_name: &str, path_pattern: &str) -> String {
    let path_pattern = normalize_path_pattern(path_pattern);

    format!(
        "{}/projects/{}/repos/{}/contents{}",
        PATH_PREFIX, project_name, repo_name, path_pattern
    )
}
