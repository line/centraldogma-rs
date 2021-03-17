mod client;
pub mod model;
mod services;

pub use client::{Client, Error, ProjectClient, RepoClient};
pub use services::{
    content::ContentService, project::ProjectService, repository::RepoService, watch::WatchService,
};
