mod client;
mod model;
pub mod services;
pub(crate) mod path;

pub use client::{Client, Error};
pub use model::{Author, Project, Repository, Entry};
pub use services::{project, repository};
