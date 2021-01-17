mod client;
mod model;
pub(crate) mod path;
pub mod services;

pub use client::{Client, Error};
pub use model::{Author, Entry, Project, Repository};
pub use services::{project, repository};
