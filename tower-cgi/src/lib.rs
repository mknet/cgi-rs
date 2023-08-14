//! # tower-cgi
//! This crate provides a mechanism for serving CGI requests using types that implement `tower::Service`.
//! This allows you to use existing frameworks built on tower, like `axum`, to write CGI scripts:
//!
//! ```rust
//! use axum::{routing::get, Router};
//! use tower_cgi::serve_cgi;
//!
//! #[tokio::main]
//! async fn main() {
//!     # std::env::set_var("REQUEST_METHOD", "GET");
//!     # std::env::set_var("CONTENT_LENGTH", "0");
//!     # std::env::set_var("REQUEST_URI", "/");
//!     let app = Router::new().route("/", get(|| async { "Hello, World!" }));
//!     serve_cgi(app).await.unwrap();
//! }
//! ```

use cgi_rs::{CGIError, CGIRequest, CGIResponse};
use hyper::{Body as HttpBody, Request, Response};
use snafu::ResultExt;
use std::convert::Infallible;
use std::io::Write;
use tower::{Service, ServiceExt};

/// Serve a CGI application.
///
/// Responses are emitted to stdout per the CGI RFC3875
pub async fn serve_cgi<S, B>(app: S) -> Result<()>
where
    S: Service<Request<HttpBody>, Response = Response<B>, Error = Infallible>
        + Clone
        + Send
        + 'static,
    B: hyper::body::HttpBody,
{
    serve_cgi_with_output(std::io::stdout(), app).await
}

/// Serve a CGI application.
///
/// Responses are emitted to the provided output stream.
pub async fn serve_cgi_with_output<S, B>(output: impl Write, app: S) -> Result<()>
where
    S: Service<Request<HttpBody>, Response = Response<B>, Error = Infallible>
        + Clone
        + Send
        + 'static,
    B: hyper::body::HttpBody,
{
    let request = CGIRequest::from_env()
        .and_then(Request::try_from)
        .context(error::CGIRequestParseSnafu)?;

    let response = app
        .oneshot(request)
        .await
        .expect("The Error type is Infallible, this should never fail.");

    let cgi_response: CGIResponse<B> = response.try_into().context(error::CGIResponseParseSnafu)?;
    cgi_response
        .write_response_to_output(output)
        .await
        .context(error::CGIResponseWriteSnafu)
}

mod error {
    use super::*;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum CgiServiceError {
        #[snafu(display("Failed to parse CGI HTTP request: {}", source))]
        CGIRequestParse { source: CGIError },

        #[snafu(display("Failed to convert HTTP response into CGI response: {}", source))]
        CGIResponseParse { source: CGIError },

        #[snafu(display("Failed to write CGI response: {}", source))]
        CGIResponseWrite { source: CGIError },
    }
}

pub use error::CgiServiceError;
type Result<T> = std::result::Result<T, CgiServiceError>;
