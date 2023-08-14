use axum::{routing::get, Router};
use tower_cgi::serve_cgi;

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/cgi-bin/sample-cgi-server",
        get(|| async { "Hello, World!" }),
    );

    if let Err(e) = serve_cgi(app).await {
        eprintln!("Error while serving CGI request: {}", e);
        std::process::exit(1);
    }
}
