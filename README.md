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
    let client = Client::from_token("http://localhost:36462", Some("token")).await.unwrap();
    // without token
    let client = Client::from_token("http://localhost:36462", None).await.unwrap();
    // your code ...
}
```

### Making typed API calls

Typed API calls are provided behind traits:

* [`ProjectService`](https://TODO)
* [`RepoService`](https://TODO)
* [`ContentService`](https://TODO)
* [`WatchService`](https://TODO)

```rust
use centraldogma::{Client, ContentService};

#[tokio::main]
fn main() {
    // without token
    let client = Client::from_token("http://localhost:36462", None).await.unwrap();

    let file = client
        .repo("project", "repository")
        .get_file(Revision::HEAD, Query::of_text("/a.yml"))
        .await
        .unwrap();
    // your code ...
}
```
