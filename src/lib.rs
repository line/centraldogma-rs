mod client;
mod model;
pub(crate) mod path;
pub mod services;

pub use client::{Client, Error};
pub use model::{
    Author, Change, ChangeContent, CommitDetail, CommitMessage, Entry, EntryContent, Project,
    PushResult, Query, QueryType, Repository,
};
pub use services::{content, project, repository};
