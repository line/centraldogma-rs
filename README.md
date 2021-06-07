# centraldogma-rs

Official Rust Client for [Central Dogma](https://line.github.io/centraldogma/).

Full documentation is available at https://docs.rs/centraldogma

## Getting started

### Installing

Add `centraldogma` crate and version to Cargo.toml.

```toml
centraldogma = "0.1.0"
```

#### Async support with tokio
The client uses [`reqwest`](https://crates.io/crates/reqwest) to make HTTP calls, which internally uses
the [`tokio`](https://crates.io/crates/tokio) runtime for async support. As such, you may require to take
a dependency on `tokio` in order to use the client.

```toml
tokio = { version = "1.2.0", features = ["full"] }
```

### Create a client

Create a new client to make API to CentralDogma using the `Client` struct.

```rust
use centraldogma::Client;

#[tokio::main]
fn main() {
    // with token
    let client = Client::new("http://localhost:36462", Some("token")).await.unwrap();
    // without token
    let client = Client::new("http://localhost:36462", None).await.unwrap();
    // your code ...
}
```

### Making typed API calls

Typed API calls are provided behind traits:

* [`ProjectService`](https://docs.rs/centraldogma/0.1.0/centraldogma/trait.ProjectService.html)
* [`RepoService`](https://docs.rs/centraldogma/0.1.0/centraldogma/trait.RepoService.html)
* [`ContentService`](https://docs.rs/centraldogma/0.1.0/centraldogma/trait.ContentService.html)
* [`WatchService`](https://docs.rs/centraldogma/0.1.0/centraldogma/trait.WatchService.html)

#### Examples

##### Get File

```rust
use centraldogma::{Client, ContentService};

#[tokio::main]
fn main() {
    // without token
    let client = Client::new("http://localhost:36462", None).await.unwrap();

    let file = client
        .repo("project", "repository")
        .get_file(Revision::HEAD, Query::of_text("/a.yml"))
        .await
        .unwrap();
    // your code ...
}
```

##### Push

```rust
use centraldogma::{Client, ContentService};

#[tokio::main]
fn main() {
    let client = Client::new("http://localhost:36462", None).await.unwrap();
    let changes = vec![Change {
        path: "/a.json".to_string(),
        content: ChangeContent::UpsertJson(serde_json::json!({"a":"b"})),
    }];
    let result = client
        .repo("foo", "bar")
        .push(
            Revision::HEAD,
            CommitMessage::only_summary("Add a.json"),
            changes,
        )
        .await
        .unwrap();
```

##### Watch file change

```rust
use centraldogma::{Client, WatchService};

#[tokio::main]
fn main() {
    let client = Client::new("http://localhost:36462", None).await.unwrap();
    let stream = client
        .repo("foo", "bar")
        .watch_file_stream(&Query::identity("/a.json").unwrap())
        .unwrap();

    tokio::spawn(async move {
        while let Some(result) = stream.next().await {
            // your code ...
        }
    })
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
