//! Content-related APIs
use crate::{
    model::{Change, Commit, CommitMessage, Entry, ListEntry, PushResult, Query, Revision},
    services::{path, status_unwrap},
    Error, RepoClient,
};

use async_trait::async_trait;
use reqwest::{Body, Method};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Push {
    commit_message: CommitMessage,
    changes: Vec<Change>,
}

/// Content-related APIs
#[async_trait]
pub trait ContentService {
    /// Retrieves the list of the files at the specified [`Revision`] matched by the path pattern.
    ///
    /// A path pattern is a variant of glob:
    ///   * `"/**"` - find all files recursively
    ///   * `"*.json"` - find all JSON files recursively
    ///   * `"/foo/*.json"` - find all JSON files under the directory /foo
    ///   * `"/*/foo.txt"` - find all files named foo.txt at the second depth level
    ///   * `"*.json,/bar/*.txt"` - use comma to specify more than one pattern.
    ///   A file will be matched if any pattern matches.
    async fn list_files(
        &self,
        revision: Revision,
        path_pattern: &str,
    ) -> Result<Vec<ListEntry>, Error>;

    /// Queries a file at the specified [`Revision`] and path with the specified [`Query`].
    async fn get_file(&self, revision: Revision, query: &Query) -> Result<Entry, Error>;

    /// Retrieves the files at the specified [`Revision`] matched by the path pattern.
    ///
    /// A path pattern is a variant of glob:
    ///   * `"/**"` - find all files recursively
    ///   * `"*.json"` - find all JSON files recursively
    ///   * `"/foo/*.json"` - find all JSON files under the directory /foo
    ///   * `"/*/foo.txt"` - find all files named foo.txt at the second depth level
    ///   * `"*.json,/bar/*.txt"` - use comma to specify more than one pattern.
    ///   A file will be matched if any pattern matches.
    async fn get_files(&self, revision: Revision, path_pattern: &str) -> Result<Vec<Entry>, Error>;

    /// Retrieves the history of the repository of the files matched by the given
    /// path pattern between two [`Revision`]s.
    /// Note that this method does not retrieve the diffs but only metadata about the changes.
    /// Use [get_diff](#tymethod.get_diff) or
    /// [get_diffs](#tymethod.get_diffs) to retrieve the diffs
    async fn get_history(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        path: &str,
        max_commits: u32,
    ) -> Result<Vec<Commit>, Error>;

    /// Returns the diff of a file between two [`Revision`]s.
    async fn get_diff(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        query: &Query,
    ) -> Result<Change, Error>;

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
    async fn get_diffs(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        path_pattern: &str,
    ) -> Result<Vec<Change>, Error>;


    /// Pushes the specified [`Change`]s to the repository.
    async fn push(
        &self,
        base_revision: Revision,
        cm: CommitMessage,
        changes: Vec<Change>,
    ) -> Result<PushResult, Error>;
}

#[async_trait]
impl<'a> ContentService for RepoClient<'a> {
    async fn list_files(
        &self,
        revision: Revision,
        path_pattern: &str,
    ) -> Result<Vec<ListEntry>, Error> {
        let req = self.client.new_request(
            Method::GET,
            path::list_contents_path(self.project, self.repo, revision, &path_pattern),
            None,
        )?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn get_file(&self, revision: Revision, query: &Query) -> Result<Entry, Error> {
        let p = path::content_path(self.project, self.repo, revision, query);
        let req = self.client.new_request(Method::GET, p, None)?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn get_files(&self, revision: Revision, path_pattern: &str) -> Result<Vec<Entry>, Error> {
        let req = self.client.new_request(
            Method::GET,
            path::contents_path(self.project, self.repo, revision, &path_pattern),
            None,
        )?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn get_history(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        path: &str,
        max_commits: u32,
    ) -> Result<Vec<Commit>, Error> {
        let p = path::content_commits_path(
            self.project,
            self.repo,
            from_rev,
            to_rev,
            path,
            max_commits,
        );
        let req = self.client.new_request(Method::GET, p, None)?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn get_diff(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        query: &Query,
    ) -> Result<Change, Error> {
        let p = path::content_compare_path(self.project, self.repo, from_rev, to_rev, query);
        let req = self.client.new_request(Method::GET, p, None)?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn get_diffs(
        &self,
        from_rev: Revision,
        to_rev: Revision,
        path_pattern: &str,
    ) -> Result<Vec<Change>, Error> {
        let p =
            path::contents_compare_path(self.project, self.repo, from_rev, to_rev, path_pattern);
        let req = self.client.new_request(Method::GET, p, None)?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    async fn push(
        &self,
        base_revision: Revision,
        cm: CommitMessage,
        changes: Vec<Change>,
    ) -> Result<PushResult, Error> {
        if cm.summary.is_empty() {
            return Err(Error::InvalidParams(
                "summary of commit_message cannot be empty",
            ));
        }
        if changes.is_empty() {
            return Err(Error::InvalidParams("no changes to commit"));
        }

        let body: String = serde_json::to_string(&Push {
            commit_message: cm,
            changes,
        })?;
        let body = Body::from(body);

        let p = path::contents_push_path(self.project, self.repo, base_revision);
        let req = self.client.new_request(Method::POST, p, Some(body))?;

        let resp = self.client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        model::{Author, ChangeContent, EntryContent, EntryType, Revision},
        Client,
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path, query_param, header, body_json},
    };

    #[tokio::test]
    async fn test_list_files() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"[
                {"path":"/a.json", "type":"JSON"},
                {"path":"/b.txt", "type":"TEXT"}
            ]"#, "application/json");
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/list/**"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let entries = client
            .repo("foo", "bar")
            .list_files(Revision::HEAD, "/**")
            .await
            .unwrap();

        server.reset().await;
        let expected = [("/a.json", EntryType::Json), ("/b.txt", EntryType::Text)];

        for (p, e) in entries.iter().zip(expected.iter()) {
            assert_eq!(p.path, e.0);
            assert_eq!(p.r#type, e.1);
        }
    }

    #[tokio::test]
    async fn test_list_files_with_revision() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"[
                {"path":"/a.json", "type":"JSON"},
                {"path":"/b.txt", "type":"TEXT"}
            ]"#, "application/json");
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/list/**"))
            .and(query_param("revision", "2"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let entries = client
            .repo("foo", "bar")
            .list_files(Revision::from(2), "/**")
            .await
            .unwrap();

        server.reset().await;
        let expected = [("/a.json", EntryType::Json), ("/b.txt", EntryType::Text)];

        for (p, e) in entries.iter().zip(expected.iter()) {
            assert_eq!(p.path, e.0);
            assert_eq!(p.r#type, e.1);
        }
    }

    #[tokio::test]
    async fn test_get_file() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"{
                    "path":"/b.txt",
                    "type":"TEXT",
                    "revision":2,
                    "url": "/api/v1/projects/foo/repos/bar/contents/b.txt",
                    "content":"hello world~!"
            }"#, "application/json");
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/contents/b.txt"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let entry = client
            .repo("foo", "bar")
            .get_file(Revision::HEAD, &Query::identity("/b.txt").unwrap())
            .await
            .unwrap();

        server.reset().await;
        assert_eq!(entry.path, "/b.txt");
        assert!(matches!(entry.content, EntryContent::Text(t) if t == "hello world~!"));
    }

    #[tokio::test]
    async fn test_get_file_text_with_escape() {
        let server = MockServer::start().await;
        let content = "foo\nb\"rb\\z";
        let resp = ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "path":"/b.txt",
                "type":"TEXT",
                "revision":2,
                "url": "/api/v1/projects/foo/repos/bar/contents/b.txt",
                "content":content
            }));
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/contents/b.txt"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let entry = client
            .repo("foo", "bar")
            .get_file(Revision::HEAD, &Query::identity("/b.txt").unwrap())
            .await
            .unwrap();

        server.reset().await;
        assert_eq!(entry.path, "/b.txt");
        assert!(matches!(entry.content, EntryContent::Text(t) if t == content));
    }

    #[tokio::test]
    async fn test_get_file_json() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"{
                    "path":"/a.json",
                    "type":"JSON",
                    "revision":2,
                    "url": "/api/v1/projects/foo/repos/bar/contents/a.json",
                    "content":{"a":"b"}
                }"#, "application/json");
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/contents/a.json"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let entry = client
            .repo("foo", "bar")
            .get_file(Revision::HEAD, &Query::identity("/a.json").unwrap())
            .await
            .unwrap();

        server.reset().await;
        assert_eq!(entry.path, "/a.json");
        let expected = serde_json::json!({"a": "b"});
        assert!(matches!(entry.content, EntryContent::Json(js) if js == expected));
    }

    #[tokio::test]
    async fn test_get_file_json_path() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"{
                    "path":"/a.json",
                    "type":"JSON",
                    "revision":2,
                    "url": "/api/v1/projects/foo/repos/bar/contents/a.json",
                    "content":"b"
                }"#, "application/json");
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/contents/a.json"))
            .and(query_param("jsonpath", "$.a"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let query = Query::of_json_path("/a.json", vec!["$.a".to_string()]).unwrap();
        let entry = client
            .repo("foo", "bar")
            .get_file(Revision::HEAD, &query)
            .await
            .unwrap();

        server.reset().await;
        assert_eq!(entry.path, "/a.json");
        let expected = serde_json::json!("b");
        assert!(matches!(entry.content, EntryContent::Json(js) if js == expected));
    }

    #[tokio::test]
    async fn test_get_file_json_path_and_revision() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"{
                    "path":"/a.json",
                    "type":"JSON",
                    "revision":2,
                    "url": "/api/v1/projects/foo/repos/bar/contents/a.json",
                    "content":"b"
                }"#, "application/json");
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/contents/a.json"))
            .and(query_param("revision", "5"))
            .and(query_param("jsonpath", "$.a"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let query = Query::of_json_path("/a.json", vec!["$.a".to_string()]).unwrap();
        let entry = client
            .repo("foo", "bar")
            .get_file(Revision::from(5), &query)
            .await
            .unwrap();

        server.reset().await;
        assert_eq!(entry.path, "/a.json");
        let expected = serde_json::json!("b");
        assert!(matches!(entry.content, EntryContent::Json(js) if js == expected));
    }

    #[tokio::test]
    async fn test_get_files() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"[{
                    "path":"/a.json",
                    "type":"JSON",
                    "revision":2,
                    "url": "/api/v1/projects/foo/repos/bar/contents/a.json",
                    "content":{"a":"b"}
                }, {
                    "path":"/b.txt",
                    "type":"TEXT",
                    "revision":2,
                    "url": "/api/v1/projects/foo/repos/bar/contents/b.txt",
                    "content":"hello world~!"
                }]"#, "application/json");
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/contents/**"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let entries = client
            .repo("foo", "bar")
            .get_files(Revision::HEAD, "/**")
            .await
            .unwrap();

        server.reset().await;
        let expected = [
            ("/a.json", EntryContent::Json(serde_json::json!({"a":"b"}))),
            ("/b.txt", EntryContent::Text("hello world~!".to_string())),
        ];

        for (p, e) in entries.iter().zip(expected.iter()) {
            assert_eq!(p.path, e.0);
            assert_eq!(p.content, e.1);
        }
    }

    #[tokio::test]
    async fn test_get_history() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"[{
                "revision":1,
                "author":{"name":"minux", "email":"minux@m.x"},
                "commitMessage":{"summary":"Add a.json"}
            }, {
                "revision":2,
                "author":{"name":"minux", "email":"minux@m.x"},
                "commitMessage":{"summary":"Edit a.json"}
            }]"#, "application/json");
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/commits/-2"))
            .and(query_param("to", "-1"))
            .and(query_param("maxCommits", "2"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let commits = client
            .repo("foo", "bar")
            .get_history(Revision::from(-2), Revision::HEAD, "/**", 2)
            .await
            .unwrap();

        let expected = [
            (
                1,
                Author {
                    name: "minux".to_string(),
                    email: "minux@m.x".to_string(),
                },
                CommitMessage {
                    summary: "Add a.json".to_string(),
                    detail: None,
                },
            ),
            (
                2,
                Author {
                    name: "minux".to_string(),
                    email: "minux@m.x".to_string(),
                },
                CommitMessage {
                    summary: "Edit a.json".to_string(),
                    detail: None,
                },
            ),
        ];

        server.reset().await;
        for (p, e) in commits.iter().zip(expected.iter()) {
            assert_eq!(p.revision.as_i64(), e.0);
            assert_eq!(p.author, e.1);
            assert_eq!(p.commit_message, e.2);
        }
    }

    #[tokio::test]
    async fn test_get_diff() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"{
                "path":"/a.json",
                "type":"APPLY_JSON_PATCH",
                "content":[{
                    "op":"safeReplace",
                    "path":"",
                    "oldValue":"bar",
                    "value":"baz"
                }]
            }"#, "application/json");
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/compare"))
            .and(query_param("from", "3"))
            .and(query_param("to", "4"))
            .and(query_param("path", "/a.json"))
            .and(query_param("jsonpath", "$.a"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let query = Query::of_json_path("/a.json", vec!["$.a".to_string()]).unwrap();
        let change = client
            .repo("foo", "bar")
            .get_diff(Revision::from(3), Revision::from(4), &query)
            .await
            .unwrap();

        let expected = Change {
            path: "/a.json".to_string(),
            content: ChangeContent::ApplyJsonPatch(serde_json::json!([{
                "op": "safeReplace",
                "path": "",
                "oldValue": "bar",
                "value": "baz"
            }])),
        };

        server.reset().await;
        assert_eq!(change, expected);
    }

    #[tokio::test]
    async fn test_get_diffs() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"[{
                "path":"/a.json",
                "type":"APPLY_JSON_PATCH",
                "content":[{
                    "op":"safeReplace",
                    "path":"",
                    "oldValue":"bar",
                    "value":"baz"
                }]
            }, {
                "path":"/b.txt",
                "type":"APPLY_TEXT_PATCH",
                "content":"--- /b.txt\n+++ /b.txt\n@@ -1,1 +1,1 @@\n-foo\n+bar"
            }]"#, "application/json");
        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/compare"))
            .and(query_param("from", "1"))
            .and(query_param("to", "4"))
            .and(query_param("pathPattern", "/**"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let changes = client
            .repo("foo", "bar")
            .get_diffs(Revision::from(1), Revision::from(4), "/**")
            .await
            .unwrap();

        let expected = [
            Change {
                path: "/a.json".to_string(),
                content: ChangeContent::ApplyJsonPatch(serde_json::json!([{
                    "op": "safeReplace",
                    "path": "",
                    "oldValue": "bar",
                    "value": "baz"
                }])),
            },
            Change {
                path: "/b.txt".to_string(),
                content: ChangeContent::ApplyTextPatch(
                    "--- /b.txt\n+++ /b.txt\n@@ -1,1 +1,1 @@\n-foo\n+bar".to_string(),
                ),
            },
        ];

        server.reset().await;
        for (c, e) in changes.iter().zip(expected.iter()) {
            assert_eq!(c, e);
        }
    }

    #[tokio::test]
    async fn test_push() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"{
                "revision":2,
                "pushedAt":"2017-05-22T00:00:00Z"
            }"#, "application/json");

        let changes = vec![Change {
            path: "/a.json".to_string(),
            content: ChangeContent::UpsertJson(serde_json::json!({"a":"b"})),
        }];
        let body = Push {
            commit_message: CommitMessage::only_summary("Add a.json"),
            changes,
        };
        Mock::given(method("POST"))
            .and(path("/api/v1/projects/foo/repos/bar/contents"))
            .and(query_param("revision", "-1"))
            .and(body_json(body))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let changes = vec![Change {
            path: "/a.json".to_string(),
            content: ChangeContent::UpsertJson(serde_json::json!({"a":"b"})),
        }];
        let result = client
            .repo("foo", "bar")
            .push(
                Revision::HEAD,
                CommitMessage::only_summary("Add a.json"),
                changes,
            )
            .await;

        let expected = PushResult {
            revision: Revision::from(2),
            pushed_at: Some("2017-05-22T00:00:00Z".to_string()),
        };

        drop(server);
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn test_push_two_files() {
        let server = MockServer::start().await;
        let resp = ResponseTemplate::new(200)
            .set_body_raw(r#"{
                "revision":3,
                "pushedAt":"2017-05-22T00:00:00Z"
            }"#, "application/json");

        let changes = vec![
            Change {
                path: "/a.json".to_string(),
                content: ChangeContent::UpsertJson(serde_json::json!({"a":"b"})),
            },
            Change {
                path: "/b.txt".to_string(),
                content: ChangeContent::UpsertText("myContent".to_string()),
            },
        ];
        let body = Push {
            commit_message: CommitMessage::only_summary("Add a.json and b.txt"),
            changes,
        };
        Mock::given(method("POST"))
            .and(path("/api/v1/projects/foo/repos/bar/contents"))
            .and(query_param("revision", "-1"))
            .and(body_json(body))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let changes = vec![
            Change {
                path: "/a.json".to_string(),
                content: ChangeContent::UpsertJson(serde_json::json!({"a":"b"})),
            },
            Change {
                path: "/b.txt".to_string(),
                content: ChangeContent::UpsertText("myContent".to_string()),
            },
        ];
        let result = client
            .repo("foo", "bar")
            .push(
                Revision::HEAD,
                CommitMessage::only_summary("Add a.json and b.txt"),
                changes,
            )
            .await;

        let expected = PushResult {
            revision: Revision::from(3),
            pushed_at: Some("2017-05-22T00:00:00Z".to_string()),
        };

        drop(server);
        assert_eq!(result.unwrap(), expected);
    }
}
