pub mod content;
mod path;
pub mod project;
pub mod repository;
pub mod watch;

use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorMessage {
    message: String,
}

/// convert HTTP Response with status < 200 and > 300 to Error
pub(crate) async fn status_unwrap(resp: Response) -> Result<Response, Error> {
    match resp.status().as_u16() {
        code if !(200..300).contains(&code) => {
            let err_body = resp.text().await?;
            let err_msg: ErrorMessage =
                serde_json::from_str(&err_body).unwrap_or(ErrorMessage { message: err_body });

            Err(Error::ErrorResponse(code, err_msg.message))
        }
        _ => Ok(resp),
    }
}
