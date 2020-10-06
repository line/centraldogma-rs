use serde::{Deserialize, Serialize};

use crate::{client, Client};
use reqwest::Method;

const PROJECT_SERVICE_PATH: &str = "projects";

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub creator: Author,
    pub url: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorMessage {
    message: String,
}

impl Project {
    pub async fn list(client: &Client) -> Result<Vec<Project>, client::Error> {
        let req = client.new_request(Method::GET, PROJECT_SERVICE_PATH, None)?;
        let resp = client.request(req).await?;

        match resp.status().as_u16() {
            code if code < 200 || code >= 300 => {
                let err_msg = resp.json::<ErrorMessage>().await?;

                Err(client::Error::ErrorResponse(code, err_msg.message))
            }
            _ => {
                let result = resp.json().await?;

                Ok(result)
            }
        }
    }
}
