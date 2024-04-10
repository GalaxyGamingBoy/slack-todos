use axum::{routing::get, Router};

pub fn get_router() -> Router {
    Router::new().route("/", get(root))
}

async fn root() -> &'static str {
    "Hello, Slack To-Do!"
}
