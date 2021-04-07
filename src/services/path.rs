use std::borrow::Cow;

use crate::model::{Query, QueryType, Revision};

const PATH_PREFIX: &str = "/api/v1";

mod params {
    pub const REVISION: &str = "revision";
    pub const JSONPATH: &str = "jsonpath";
    pub const PATH: &str = "path";
    pub const PATH_PATTERN: &str = "pathPattern";
    pub const MAX_COMMITS: &str = "maxCommits";
    pub const FROM: &str = "from";
    pub const TO: &str = "to";
}

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

    let mut s = form_urlencoded::Serializer::for_suffix(url, len);
    if let Some(v) = revision.as_ref() {
        add_pair(&mut s, params::REVISION, &v.to_string());
    }

    s.finish()
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

    let mut s = form_urlencoded::Serializer::for_suffix(url, len);
    if let Some(v) = revision.as_ref() {
        add_pair(&mut s, params::REVISION, &v.to_string());
    }

    s.finish()
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
    let mut s = form_urlencoded::Serializer::for_suffix(url, len);
    if let Some(v) = revision.as_ref() {
        add_pair(&mut s, params::REVISION, &v.to_string());
    }

    if let QueryType::JsonPath(expressions) = &query.r#type {
        for expression in expressions.iter() {
            add_pair(&mut s, params::JSONPATH, expression);
        }
    }

    s.finish()
}

pub(crate) fn content_commits_path(
    project_name: &str,
    repo_name: &str,
    from_rev: Revision,
    to_rev: Revision,
    path: &str,
    max_commits: Option<u32>,
) -> String {
    let url = format!(
        "{}/projects/{}/repos/{}/commits/{}?",
        PATH_PREFIX,
        project_name,
        repo_name,
        &from_rev.to_string(),
    );

    let len = url.len();
    let mut s = form_urlencoded::Serializer::for_suffix(url, len);
    add_pair(&mut s, params::PATH, path);

    if let Some(v) = to_rev.as_ref() {
        add_pair(&mut s, params::TO, &v.to_string());
    }

    if let Some(c) = max_commits {
        add_pair(&mut s, params::MAX_COMMITS, &c.to_string());
    }

    s.finish()
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
    let mut s = form_urlencoded::Serializer::for_suffix(url, len);
    add_pair(&mut s, params::PATH, &query.path);

    if let Some(v) = from_rev.as_ref() {
        add_pair(&mut s, params::FROM, &v.to_string());
    }
    if let Some(v) = to_rev.as_ref() {
        add_pair(&mut s, params::TO, &v.to_string());
    }

    if let QueryType::JsonPath(expressions) = &query.r#type {
        for expression in expressions.iter() {
            add_pair(&mut s, params::JSONPATH, expression);
        }
    }

    s.finish()
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
    let mut s = form_urlencoded::Serializer::for_suffix(url, len);
    add_pair(&mut s, params::PATH_PATTERN, &path_pattern);

    if let Some(v) = from_rev.as_ref() {
        add_pair(&mut s, params::FROM, &v.to_string());
    }
    if let Some(v) = to_rev.as_ref() {
        add_pair(&mut s, params::TO, &v.to_string());
    }

    s.finish()
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
    let mut s = form_urlencoded::Serializer::for_suffix(url, len);

    if let Some(v) = base_revision.as_ref() {
        add_pair(&mut s, params::REVISION, &v.to_string());
    }

    s.finish()
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
            add_pair(&mut serializer, params::JSONPATH, expression);
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

fn add_pair<'a, T>(s: &mut form_urlencoded::Serializer<'a, T>, key: &str, value: &str)
where
    T: form_urlencoded::Target
{
    if !value.is_empty() {
       s.append_pair(key, value);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_content_commits_path() {
        let full_arg_path = content_commits_path("foo", "bar", Revision::from(1), Revision::from(2), "/a.json", Some(5));
        assert_eq!(full_arg_path, "/api/v1/projects/foo/repos/bar/commits/1?path=%2Fa.json&to=2&maxCommits=5");

        let omitted_max_commmit_path = content_commits_path("foo", "bar", Revision::from(1), Revision::from(2), "/a.json", None);
        assert_eq!(omitted_max_commmit_path, "/api/v1/projects/foo/repos/bar/commits/1?path=%2Fa.json&to=2");

        let omitted_from_to_path = content_commits_path("foo", "bar", Revision::DEFAULT, Revision::DEFAULT, "/a.json", Some(5));
        assert_eq!(omitted_from_to_path, "/api/v1/projects/foo/repos/bar/commits/?path=%2Fa.json&maxCommits=5");

        let omitted_all_path = content_commits_path("foo", "bar", Revision::DEFAULT, Revision::DEFAULT, "/a.json", None);
        assert_eq!(omitted_all_path, "/api/v1/projects/foo/repos/bar/commits/?path=%2Fa.json");
    }

    #[test]
    fn test_content_compare_path() {
        let full_arg_path = content_compare_path("foo", "bar", Revision::from(1), Revision::from(2), &Query::identity("/a.json").unwrap());
        assert_eq!(full_arg_path, "/api/v1/projects/foo/repos/bar/compare?path=%2Fa.json&from=1&to=2");

        let omitted_from_path = content_compare_path("foo", "bar", Revision::DEFAULT, Revision::from(2), &Query::identity("/a.json").unwrap());
        assert_eq!(omitted_from_path, "/api/v1/projects/foo/repos/bar/compare?path=%2Fa.json&to=2");

        let omitted_to_path = content_compare_path("foo", "bar", Revision::from(1), Revision::DEFAULT, &Query::identity("/a.json").unwrap());
        assert_eq!(omitted_to_path, "/api/v1/projects/foo/repos/bar/compare?path=%2Fa.json&from=1");

        let omitted_all_path = content_compare_path("foo", "bar", Revision::DEFAULT, Revision::DEFAULT, &Query::identity("/a.json").unwrap());
        assert_eq!(omitted_all_path, "/api/v1/projects/foo/repos/bar/compare?path=%2Fa.json");

        let with_json_query = content_compare_path("foo", "bar", Revision::DEFAULT, Revision::DEFAULT, &Query::of_json_path("/a.json", vec!["a".to_string()]).unwrap());
        assert_eq!(with_json_query, "/api/v1/projects/foo/repos/bar/compare?path=%2Fa.json&jsonpath=a");
    }
}
