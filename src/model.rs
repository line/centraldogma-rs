use serde::{Deserialize, Serialize};

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
    pub head_revision: Option<u64>,
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
    pub revision: Option<u64>,
    pub url: Option<String>,
    pub modified_at: Option<String>,
}

#[derive(Debug)]
pub enum QueryType {
    Identity,
    JsonPath(Vec<String>),
}

pub struct Query {
    pub path: String,
    pub r#type: QueryType,
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
    pub revision: i64,
    pub pushed_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub revision: i64,
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
