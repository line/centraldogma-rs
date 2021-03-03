//! Content-related APIs
use crate::{
    client::status_unwrap,
    model::{Change, Commit, CommitMessage, Entry, ListEntry, PushResult, Query, Revision},
    path, Error, RepoClient,
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
