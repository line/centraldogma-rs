//! Data models of CentralDogma
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// A revision number of a [`Commit`].
///
/// A revision number is an integer which refers to a specific point of repository history.
/// When a repository is created, it starts with an initial commit whose revision is 1.
/// As new commits are added, each commit gets its own revision number,
/// monotonically increasing from the previous commit's revision. i.e. 1, 2, 3, ...
///
/// A revision number can also be represented as a negative integer.
/// When a revision number is negative, we start from -1 which refers to the latest commit in repository history,
/// which is often called 'HEAD' of the repository.
/// A smaller revision number refers to the older commit.
/// e.g. -2 refers to the commit before the latest commit, and so on.
///
/// A revision with a negative integer is called 'relative revision'.
/// By contrast, a revision with a positive integer is called 'absolute revision'.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Revision(i64);

impl Revision {
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Revision {
    /// Revision `-1`, also known as `HEAD`.
    pub const HEAD: Revision = Revision(-1);
    /// Revision `1`, also known as `INIT`.
    pub const INIT: Revision = Revision(1);

    /// Create a new instance with the specified revision number.
    pub fn from(i: i64) -> Self {
        Revision(i)
    }
}

/// Creator of a project or repository or commit
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    /// Name of this author.
    pub name: String,
    /// Email of this author.
    pub email: String,
}

/// A top-level element in Central Dogma storage model.
/// A project has "dogma" and "meta" repositories by default which contain project configuration
/// files accessible by administrators and project owners respectively.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Name of this project.
    pub name: String,
    /// The author who initially created this project.
    pub creator: Author,
    /// Url of this project
    pub url: Option<String>,
    /// When the project was created
    pub created_at: Option<String>,
}

/// Repository information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    /// Name of this repository.
    pub name: String,
    /// The author who initially created this repository.
    pub creator: Author,
    /// Head [`Revision`] of the repository.
    pub head_revision: Revision,
    /// Url of this repository.
    pub url: Option<String>,
    /// When the repository was created.
    pub created_at: Option<String>,
}

/// The content of an [`Entry`]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type", content = "content")]
pub enum EntryContent {
    /// Content as a JSON Value.
    Json(serde_json::Value),
    /// Content as a String.
    Text(String),
    /// This Entry is a directory.
    Directory,
}

/// A file or a directory in a repository.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Path of this entry.
    pub path: String,
    /// Content of this entry.
    #[serde(flatten)]
    pub content: EntryContent,
    /// Revision of this entry.
    pub revision: Revision,
    /// Url of this entry.
    pub url: String,
    /// When this entry was last modified.
    pub modified_at: Option<String>,
}

impl Entry {
    pub fn entry_type(&self) -> EntryType {
        match self.content {
            EntryContent::Json(_) => EntryType::Json,
            EntryContent::Text(_) => EntryType::Text,
            EntryContent::Directory => EntryType::Directory,
        }
    }
}

/// The type of a [`ListEntry`]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntryType {
    /// A UTF-8 encoded JSON file.
    Json,
    /// A UTF-8 encoded text file.
    Text,
    /// A directory.
    Directory,
}

/// A metadata of a file or a directory in a repository.
/// ListEntry has no content.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListEntry {
    pub path: String,
    pub r#type: EntryType,
}

/// Type of a [`Query`]
#[derive(Debug)]
pub enum QueryType {
    Identity,
    IdentityJson,
    IdentityText,
    JsonPath(Vec<String>),
}

/// A Query on a file
#[derive(Debug)]
pub struct Query {
    pub(crate) path: String,
    pub(crate) r#type: QueryType,
}

impl Query {
    fn normalize_path(path: &str) -> String {
        if path.starts_with('/') {
            path.to_owned()
        } else {
            format!("/{}", path)
        }
    }

    /// Returns a newly-created [`Query`] that retrieves the content as it is.
    /// Returns `None` if path is empty
    pub fn identity(path: &str) -> Option<Self> {
        if path.is_empty() {
            return None;
        }
        Some(Query {
            path: Self::normalize_path(path),
            r#type: QueryType::Identity,
        })
    }

    /// Returns a newly-created [`Query`] that retrieves the textual content as it is.
    /// Returns `None` if path is empty
    pub fn of_text(path: &str) -> Option<Self> {
        if path.is_empty() {
            return None;
        }
        Some(Query {
            path: Self::normalize_path(path),
            r#type: QueryType::IdentityText,
        })
    }

    /// Returns a newly-created [`Query`] that retrieves the JSON content as it is.
    /// Returns `None` if path is empty
    pub fn of_json(path: &str) -> Option<Self> {
        if path.is_empty() {
            return None;
        }
        Some(Query {
            path: Self::normalize_path(path),
            r#type: QueryType::IdentityJson,
        })
    }

    /// Returns a newly-created [`Query`] that applies a series of
    /// [JSON path expressions](https://github.com/json-path/JsonPath/blob/master/README.md)
    /// to the content.
    /// Returns `None` if path is empty or does not end with `.json`.
    pub fn of_json_path(path: &str, exprs: Vec<String>) -> Option<Self> {
        if !path.to_lowercase().ends_with("json") {
            return None;
        }
        Some(Query {
            path: Self::normalize_path(path),
            r#type: QueryType::JsonPath(exprs),
        })
    }
}

/// Typed content of a [`CommitMessage`]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "markup", content = "detail")]
pub enum CommitDetail {
    /// Commit details as markdown
    Markdown(String),
    /// Commit details as plaintext
    Plaintext(String),
}

/// Description of a [`Commit`]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommitMessage {
    /// Summary of this commit message
    pub summary: String,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    /// Detailed description of this commit message
    pub detail: Option<CommitDetail>,
}

/// Result of a [push](trait@crate::ContentService#tymethod.push) operation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushResult {
    /// Revision of this commit.
    pub revision: Revision,
    /// When this commit was pushed.
    pub pushed_at: Option<String>,
}

/// A set of Changes and its metadata.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    /// Revision of this commit.
    pub revision: Revision,
    /// Author of this commit.
    pub author: Author,
    /// Description of this commit.
    pub commit_message: CommitMessage,
    /// When this commit was pushed.
    pub pushed_at: Option<String>,
}

/// Typed content of a [`Change`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type", content = "content")]
pub enum ChangeContent {
    /// Adds a new JSON file or replaces an existing file with the provided json.
    UpsertJson(serde_json::Value),

    /// Adds a new text file or replaces an existing file with the provided content.
    UpsertText(String),

    /// Removes an existing file.
    Remove,

    /// Renames an existsing file to this provided path.
    Rename(String),

    /// Applies a JSON patch to a JSON file with the provided JSON patch object,
    /// as defined in [RFC 6902](https://tools.ietf.org/html/rfc6902).
    ApplyJsonPatch(serde_json::Value),

    /// Applies a textual patch to a text file with the provided
    /// [unified format](https://en.wikipedia.org/wiki/Diff_utility#Unified_format) string.
    ApplyTextPatch(String),
}

/// A modification of an individual [`Entry`]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    /// Path of the file change.
    pub path: String,
    /// Content of the file change.
    #[serde(flatten)]
    pub content: ChangeContent,
}

/// A change result from a
/// [watch_file](trait@crate::WatchService#tymethod.watch_file_stream) operation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchFileResult {
    /// Revision of the change.
    pub revision: Revision,
    /// Content of the change.
    pub entry: Entry,
}

/// A change result from a
/// [watch_repo](trait@crate::WatchService#tymethod.watch_repo_stream) operation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchRepoResult {
    /// Revision of the change.
    pub revision: Revision,
}

pub(crate) trait Watchable: DeserializeOwned + Send {
    fn revision(&self) -> Revision;
}

impl Watchable for WatchFileResult {
    fn revision(&self) -> Revision {
        self.revision
    }
}

impl Watchable for WatchRepoResult {
    fn revision(&self) -> Revision {
        self.revision
    }
}
