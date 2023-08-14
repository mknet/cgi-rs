# What is this?

This repo contains utilities for writing Rust packages which adhere to [Common Gateway Interface](https://en.wikipedia.org/wiki/Common_Gateway_Interface) (CGI).
A CGI server will exec a CGI script to handle requests that are routed to it, forking a new process for each request.
This technology's role in implementing commont HTTP services has been succeeded by technologies which focus on keeping service processes running, improving latency and throughput.
Despite this, CGI is still useful in cases optimizing for low ambient resource utilization at the expense of some throughput.

## tower-cgi
This crate provides a mechanism for serving CGI requests using types that implement [`tower::Service`](https://docs.rs/tower/latest/tower/trait.Service.html).
This allows you to use existing frameworks built on tower, like [`axum`](https://crates.io/crates/axum), to write CGI scripts:

```rust
use axum::{routing::get, Router};
use tower_cgi::serve_cgi;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    serve_cgi(app).await.unwrap();
}
```

## cgi-rs
This module provides mechanisms for mapping a CGI request to [`hyper::Request`](https://docs.rs/hyper/latest/hyper/struct.Request.html)/[`hyper::Response`](https://docs.rs/hyper/latest/hyper/struct.Response.html) types, which are used by
many HTTP implementations.

### Current limitations:
* Only provides the needed utilities to create CGI scripts, not CGI servers.
* Only "Document"-type responses are supported.
* Only a subset of the CGI environment variables are hoisted into Requests.
* Does not support Windows.

### Examples
#### Parsing an HTTP Request
```rust
use hyper::{Request, Body};
use cgi_rs::CGIRequest;

fn main() {
    // In a CGI environment, the CGI server would set these variables, as well as others.
    std::env::set_var("REQUEST_METHOD", "GET");
    std::env::set_var("CONTENT_LENGTH", "0");
    std::env::set_var("REQUEST_URI", "/");

    let cgi_request: Request<Body> = CGIRequest::from_env()
        .and_then(Request::try_from).unwrap();
}
```

#### Querying for Additional CGI Variables
It's simple enough to fetch environment variables, but `cgi-rs` provides a convenient way to fetch and parse
CGI environment variables (referred to as "meta-variables" by RFC3875)

```rust
fn main() {
    let method = cgi_rs::MetaVariableKind::RequestMethod.try_from_env().unwrap();
    assert_eq!(method.as_str().unwrap(), "GET");
}
```
