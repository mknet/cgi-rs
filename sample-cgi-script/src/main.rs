use axum::{routing::get, Router};
use axum::http::StatusCode;
use axum::response::Response;
use tower_cookies::{Cookie, Cookies};
use tower_sessions::cookie::time::Duration;
use tower_sessions::{MemoryStore, Session, SessionStore};
use tower_sessions::session::Record;
use tower_cgi::serve_cgi;

use tower_sessions_file_based_store::FileStore;

#[tokio::main]
async fn main() {
    let session_store = FileStore::new("./", "prefix-", ".json");
    // let session_store = MemoryStore::default();
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false)
        //.with_always_save(true)
        .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::seconds(15)));

    let app = Router::new().route(
        "/cgi-bin/sample-cgi-server/",
        get(|cookies: Cookies, session: Session| async move  {
            cookies.add(Cookie::new("hello_world", "hello_world"));
            session.clear().await;
            session.insert("foo", "bar").await.unwrap();
            let value: String = session.get("foo").await.unwrap().unwrap_or("no value".to_string());

            value

        }),
    ).route(
        "/cgi-bin/sample-cgi-server/with/path-info",
        get(|| async { "Hello, PATH_INFO" }),
    ).layer(session_layer);

    if let Err(e) = serve_cgi(app).await {
        eprintln!("Error while serving CGI request: {}", e);
        std::process::exit(1);
    }
}
