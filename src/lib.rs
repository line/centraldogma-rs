mod client;
mod model;
pub(crate) mod path;
pub mod services;

pub use client::{
    Client, ContentService, Error, ProjectService, RepoClient, RepoService, WatchService,
};
pub use model::{
    Author, Change, ChangeContent, Commit, CommitDetail, CommitMessage, Entry, EntryContent,
    Project, PushResult, Query, QueryType, Repository, Revision, WatchResult,
};
pub use services::{content, project, repository, watch};
