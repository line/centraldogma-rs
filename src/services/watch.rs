//! Watch-related APIs
use std::{pin::Pin, time::Duration};

use crate::{
    model::{Query, Revision, WatchFileResult, WatchRepoResult, Watchable},
    services::{path, status_unwrap},
    Client, Error, RepoClient,
};

use futures::{Stream, StreamExt};
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

/// Watch-related APIs
pub trait WatchService {
    /// Returns a stream which output a [`WatchFileResult`] when the result of the
    /// given [`Query`] becomes available or changes
    fn watch_file_stream(
        &self,
        query: &Query,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchFileResult> + Send>>, Error>;

    /// Returns a stream which output a [`WatchRepoResult`] when the repository has a new commit
    /// that contains the changes for the files matched by the given `path_pattern`.
    fn watch_repo_stream(
        &self,
        path_pattern: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchRepoResult> + Send>>, Error>;
}

impl<'a> WatchService for RepoClient<'a> {
    fn watch_file_stream(
        &self,
        query: &Query,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchFileResult> + Send>>, Error> {
        let p = path::content_watch_path(self.project, self.repo, query);

        Ok(watch_stream(self.client.clone(), p).boxed())
    }

    fn watch_repo_stream(
        &self,
        path_pattern: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = WatchRepoResult> + Send>>, Error> {
        let p = path::repo_watch_path(self.project, self.repo, path_pattern);

        Ok(watch_stream(self.client.clone(), p).boxed())
    }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;
    use crate::model::{Entry, EntryContent};
    use wiremock::{Mock, MockServer, Respond, ResponseTemplate, matchers::{method, path, header}};

    struct MockResponse {
        first_time: AtomicBool
    }

    impl Respond for MockResponse {
        fn respond(&self, _req: &wiremock::Request) -> ResponseTemplate {
            if self.first_time.swap(false, Ordering::SeqCst) {
                println!("Called 1");
                ResponseTemplate::new(304)
                    .set_delay(Duration::from_millis(100))
            } else {
                println!("Called 2");
                let resp = r#"{
                    "revision":3,
                    "entry":{
                        "path":"/a.json",
                        "type":"JSON",
                        "content": {"a":"b"},
                        "revision":3,
                        "url": "/api/v1/projects/foo/repos/bar/contents/a.json"
                    }
                }"#;
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(100))
                    .set_body_raw(resp, "application/json")
            }
        }
    }

    #[tokio::test]
    async fn test_watch_file() {
        let server = MockServer::start().await;
        let resp = MockResponse { first_time: AtomicBool::new(true) };

        Mock::given(method("GET"))
            .and(path("/api/v1/projects/foo/repos/bar/contents/a.json"))
            .and(header("if-none-match", "-1"))
            .and(header("prefer", "wait=60"))
            .and(header("Authorization", "Bearer anonymous"))
            .respond_with(resp)
            .expect(2)
            .mount(&server)
            .await;

        let client = Client::new(&server.uri(), None).await.unwrap();
        let stream = client
            .repo("foo", "bar")
            .watch_file_stream(&Query::identity("/a.json").unwrap())
            .unwrap()
            .take_until(tokio::time::sleep(Duration::from_secs(3)));
        tokio::pin!(stream);

        let result = stream.next().await;

        server.reset().await;
        let result = result.unwrap();
        assert_eq!(result.revision, Revision::from(3));
        assert_eq!(
            result.entry,
            Entry {
                path: "/a.json".to_string(),
                content: EntryContent::Json(serde_json::json!({"a":"b"})),
                revision: Revision::from(3),
                url: "/api/v1/projects/foo/repos/bar/contents/a.json".to_string(),
                modified_at: None,
            }
        );
    }
}
