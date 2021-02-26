use std::time::Duration;

use crate::{
    client::status_unwrap,
    model::{Query, Revision, WatchFileResult, WatchRepoResult, Watchable},
    path, Client, Error,
};

use futures::Stream;
use reqwest::{Method, Request, StatusCode};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const DELAY_ON_SUCCESS: Duration = Duration::from_secs(1);
const MAX_FAILED_COUNT: usize = 5; // Max base wait time 2 << 5 = 64 secs
const JITTER_RATE: f32 = 0.2;

async fn request_watch<D: Watchable>(client: &Client, req: Request) -> Result<Option<D>, Error> {
    let resp = client.request(req).await?;
    if resp.status() == StatusCode::NOT_MODIFIED {
        return Ok(None);
    }
    let ok_resp = status_unwrap(resp).await?;
    let result = ok_resp.json().await?;

    Ok(Some(result))
}

fn delay_time_for(failed_count: usize) -> Duration {
    let base_time_ms = (2 << failed_count) * 1000;
    let jitter = (fastrand::f32() * JITTER_RATE * base_time_ms as f32) as u64;

    Duration::from_millis(base_time_ms + jitter)
}

struct WatchState {
    client: Client,
    path: String,
    last_known_revision: Option<Revision>,
    failed_count: usize,
    success_delay: Option<Duration>,
}

fn watch_stream<D: Watchable>(client: Client, path: String) -> impl Stream<Item = D> + Send {
    let init_state = WatchState {
        client,
        path,
        last_known_revision: None,
        failed_count: 0,
        success_delay: None,
    };
    futures::stream::unfold(init_state, |mut state| async move {
        if let Some(d) = state.success_delay.take() {
            tokio::time::sleep(d).await;
        }
        loop {
            let req = match state.client.new_watch_request(
                Method::GET,
                &state.path,
                None,
                state.last_known_revision,
                DEFAULT_TIMEOUT,
            ) {
                Ok(r) => r,
                Err(_) => {
                    return None;
                }
            };

            let resp: Result<Option<D>, _> = request_watch(&state.client, req).await;
            let next_delay = match resp {
                // Send Ok data out
                Ok(Some(watch_result)) => {
                    state.last_known_revision = Some(watch_result.revision());
                    state.failed_count = 0;
                    state.success_delay = Some(DELAY_ON_SUCCESS);

                    return Some((watch_result, state));
                }
                Ok(None) => {
                    state.failed_count = 0;
                    Duration::from_secs(1)
                }
                Err(Error::HttpClient(e)) if e.is_timeout() => Duration::from_secs(1),
                Err(e) => {
                    log::debug!("Request error: {}", e);
                    if state.failed_count < MAX_FAILED_COUNT {
                        state.failed_count += 1;
                    }
                    delay_time_for(state.failed_count)
                }
            };
            // Delay
            tokio::time::sleep(next_delay).await;
        }
    })
}

pub fn watch_file_stream(
    client: Client,
    project_name: &str,
    repo_name: &str,
    query: &Query,
) -> Result<impl Stream<Item = WatchFileResult> + Send, Error> {
    let p = path::content_watch_path(project_name, repo_name, query);

    Ok(watch_stream(client, p))
}

pub fn watch_repo_stream(
    client: Client,
    project_name: &str,
    repo_name: &str,
    path_pattern: &str,
) -> Result<impl Stream<Item = WatchRepoResult> + Send, Error> {
    let p = path::repo_watch_path(project_name, repo_name, path_pattern);

    Ok(watch_stream(client, p))
}
