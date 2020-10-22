use crate::{client, Client};

use reqwest::Method;
use reqwest::Body;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::json;

const PROJECT_SERVICE_PATH: &str = "/api/v1/projects";
const ACTION_REMOVED: &str = "removed";

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
struct ErrorMessage {
    message: String,
}

async fn status_unwrap(resp: Response) -> Result<Response, client::Error> {
    match resp.status().as_u16() {
        code if code < 200 || code >= 300 => {
            let err_msg = resp.json::<ErrorMessage>().await?;

            Err(client::Error::ErrorResponse(code, err_msg.message))
        }
        _ => {
            Ok(resp)
        }
    }
}

impl Project {
    pub async fn list(client: &Client) -> Result<Vec<Project>, client::Error> {
        let req = client.new_request(Method::GET, PROJECT_SERVICE_PATH, None)?;
        let resp = client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    /// Returns list of project name where status is removed
    pub async fn list_removed(client: &Client) -> Result<Vec<String>, client::Error> {
        #[derive(Deserialize)]
        struct RemovedProject {
            name: String
        }
        let path = format!("{}?status=removed", PROJECT_SERVICE_PATH);
        let req = client.new_request(Method::GET, path, None)?;
        let resp = client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;

        let result: Vec<RemovedProject> = ok_resp.json().await?;
        let result = result.into_iter().map(|p| p.name).collect();

        Ok(result)
    }

    pub async fn create(client: &Client, name: &str) -> Result<Project, client::Error> {
        #[derive(Serialize)]
        struct CreateProject<'a> {
            name: &'a str
        };
        let body: Vec<u8> = serde_json::to_vec(&CreateProject { name })?;
        let body = Body::from(body);
        let req = client.new_request(Method::POST, PROJECT_SERVICE_PATH, Some(body))?;

        let resp = client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    pub async fn remove(client: &Client, name: &str) -> Result<(), client::Error> {
        let path = format!("{}/{}", PROJECT_SERVICE_PATH, name);
        let req = client.new_request(Method::DELETE, path, None)?;

        let resp = client.request(req).await?;
        let _ = status_unwrap(resp).await?;

        Ok(())
    }

    pub async fn unremove(client: &Client, name: &str) -> Result<Project, client::Error> {
        let path = format!("{}/{}", PROJECT_SERVICE_PATH, name);
        let body: Vec<u8> = serde_json::to_vec(&json!([
            {"op":"replace", "path":"/status", "value":"active"}
        ]))?;
        let body = Body::from(body);
        let req = client.new_request(Method::PATCH, path, Some(body))?;

        let resp = client.request(req).await?;
        let ok_resp = status_unwrap(resp).await?;
        let result = ok_resp.json().await?;

        Ok(result)
    }

    pub async fn purge(client: &Client, name: &str) -> Result<(), client::Error> {
        let path = format!("{}/{}/{}", PROJECT_SERVICE_PATH, name, ACTION_REMOVED);
        let req = client.new_request(Method::DELETE, path, None)?;

        let resp = client.request(req).await?;
        let _ = status_unwrap(resp).await?;

        Ok(())
    }
}
