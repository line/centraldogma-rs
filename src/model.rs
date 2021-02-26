use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Revision(i64);

impl std::fmt::Display for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Revision {
    pub const HEAD: Revision = Revision(-1);

    pub fn from(i: i64) -> Self {
        Revision(i)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    pub creator: Author,
    pub url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub name: String,
    pub creator: Option<Author>,
    pub head_revision: Option<Revision>,
    pub url: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type", content = "content")]
pub enum EntryContent {
    Json(serde_json::Value),
    Text(String),
    Directory(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub path: String,
    #[serde(flatten)]
    pub content: EntryContent,
    pub revision: Option<Revision>,
    pub url: Option<String>,
    pub modified_at: Option<String>,
}

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
    /// Deprecated.  
    /// Use [`Self::of_text()`] or [`Self::of_json()`] instead
    #[deprecated]
    pub fn identity(path: &str) -> Self {
        Query {
            path: Self::normalize_path(path),
            r#type: QueryType::Identity,
        }
    }

    /// Returns a newly-created [`Query`] that retrieves the textual content as it is.
    pub fn of_text(path: &str) -> Self {
        Query {
            path: Self::normalize_path(path),
            r#type: QueryType::IdentityText
        }
    }

    /// Returns a newly-created [`Query`] that retrieves the JSON content as it is.
    pub fn of_json(path: &str) -> Self {
        Query {
            path: Self::normalize_path(path),
            r#type: QueryType::IdentityJson
        }
    }

    /// Returns a newly-created [`Query`] that applies a series of
    /// [JSON path expressions](https://github.com/json-path/JsonPath/blob/master/README.md)
    /// to the content.
    /// Returns `None` if path does not end with `.json`.
    pub fn of_json_path(path: &str, exprs: Vec<String>) -> Option<Self> {
        if !path.to_lowercase().ends_with("json") {
            return None;
        }
        Some(Query {
            path: Self::normalize_path(path),
            r#type: QueryType::JsonPath(exprs)
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "markup", content = "detail")]
pub enum CommitDetail {
    Json(serde_json::Value),
    Plaintext(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitMessage {
    pub summary: String,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub detail: Option<CommitDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushResult {
    pub revision: Revision,
    pub pushed_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub revision: Revision,
    pub author: Option<Author>,
    pub commit_message: Option<CommitMessage>,
    pub pushed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type", content = "content")]
pub enum ChangeContent {
    UpsertJson(serde_json::Value),
    UpsertText(String),
    Remove(String),
    Rename(String),
    ApplyJsonPatch(serde_json::Value),
    ApplyTextPatch(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub path: String,
    #[serde(flatten)]
    pub content: ChangeContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchResult {
    pub revision: Revision,
    pub entry: Option<Entry>,
}
