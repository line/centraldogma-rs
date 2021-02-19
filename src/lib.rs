mod client;
mod model;
pub(crate) mod path;
pub mod services;

pub use client::{RepoClient, Client, Error};
pub use model::{
    Author, Change, ChangeContent, CommitDetail, CommitMessage, Entry, EntryContent, Project,
    PushResult, Query, QueryType, Repository, Revision, WatchResult
};
pub use services::{content, project, repository, watch};
