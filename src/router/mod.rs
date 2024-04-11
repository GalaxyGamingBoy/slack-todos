use std::collections::HashMap;

use axum::{
    extract::State,
    routing::{get, post},
    Form, Router,
};

use crate::{
    slack::{SlackBlock, SlackCommand},
    todo::Todo,
    ServerState,
};

pub fn get_router() -> Router<ServerState> {
    Router::new()
        .route("/", get(root))
        .route("/todo/new", post(todo_new))
}

async fn root() -> &'static str {
    "Hello, Slack To-Do!"
}

async fn todo_new(State(state): State<ServerState>, Form(payload): Form<SlackCommand>) {
    let args: Vec<String> = payload.text.split(" ").map(|v| v.to_string()).collect();
    let mut todo = Todo {
        title: args[0].clone(),
        description: args.get(1).cloned(),
        slack_user: payload.user_id,
        ..Default::default()
    };
    todo.assign_id().insert(&state.db).await;

    let mut template: HashMap<&str, String> = HashMap::new();
    template.insert("title", todo.title);
    template.insert("desc", todo.description.unwrap_or_default());

    let mut block = SlackBlock::new("created".to_string());
    block.load().fill(template);

    state
        .slack
        .send_webhook(payload.response_url, block.into(), true)
        .await;
}
