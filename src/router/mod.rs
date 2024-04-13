use std::collections::HashMap;

use axum::{
    extract::State,
    routing::{get, post},
    Form, Router,
};

use crate::{
    slack::{
        block::SlackBlock,
        modal::SlackModal,
        payloads::{SlackCommand, SlackInteraction, SlackInteractionData},
    },
    todo::Todo,
    ServerState,
};

pub fn get_router() -> Router<ServerState> {
    Router::new()
        .route("/", get(root))
        .route("/todo/new", post(todo_new))
        .route("/slack/interactivity", post(slack_interactivity))
}

async fn root() -> &'static str {
    "Hello, Slack To-Do!"
}

async fn todo_new(State(state): State<ServerState>, Form(payload): Form<SlackCommand>) {
    if payload.text.trim().is_empty() {
        let mut template: HashMap<&str, String> = HashMap::new();
        template.insert("initial_channel", payload.channel_id);

        let mut modal = SlackModal::new("create".to_string(), payload.trigger_id);
        modal.load().fill(template);

        match state.slack.open_modal(&modal).await {
            Ok(_) => {}
            Err(err) => println!("An error occured while openning a modal. {err}"),
        }

        return;
    }

    let mut todo = Todo {
        title: payload.text,
        slack_user: payload.user_id,
        ..Default::default()
    };
    match todo.assign_id().insert(&state.db).await {
        Ok(_) => {}
        Err(err) => {
            println!("An error occured inserting todo to the database. {err}");
            return;
        }
    }

    let mut template: HashMap<&str, String> = HashMap::new();
    template.insert("title", todo.title);
    template.insert("desc", todo.description.unwrap_or_default());

    let mut block = SlackBlock::new("created".to_string());
    block.load().fill(template);

    state
        .slack
        .send_webhook(payload.response_url, &mut block.into(), true)
        .await;
}

async fn slack_interactivity(
    State(state): State<ServerState>,
    Form(interaction): Form<SlackInteraction>,
) {
    let payload = serde_json::from_str(&interaction.payload);

    let payload: SlackInteractionData = match payload {
        Ok(v) => v,
        Err(err) => {
            println!("Couldn't extract payload from slack: {err}");
            return;
        }
    };

    if payload.r#type != "view_submission" {
        return;
    };

    let title =
        &payload.view.state["values"]["input-title"]["input-title-action"]["value"].as_str();
    let description = &payload.view.state["values"]["input-description"]
        ["input-description-action"]["value"]
        .as_str();
    let channel = &payload.view.state["values"]["input-channel"]["input-channel-action"]
        ["selected_channel"]
        .as_str();

    let mut todo = Todo {
        title: title
            .expect("Slack Interaction did not contain title! Is it in the modal?")
            .to_string(),
        description: description.map(str::to_string),
        slack_user: payload.user.id.clone(),
        ..Default::default()
    };

    match todo.assign_id().insert(&state.db).await {
        Ok(_) => {}
        Err(err) => {
            println!("An error occured inserting todo to the database. {err}");
            return;
        }
    }

    let mut template: HashMap<&str, String> = HashMap::new();
    template.insert("title", todo.title);
    template.insert("desc", todo.description.unwrap_or_default());

    let mut block = SlackBlock::new("created".to_string());
    block.load().fill(template).trim();

    println!("{:?}", block.data);

    match state
        .slack
        .send_ephemeral(
            block.data,
            channel
                .expect("Slack Interaction did not contain channel-id! Is it in the modal?")
                .to_string(),
            payload.user.id,
        )
        .await
    {
        Ok(_) => {}
        Err(err) => println!("An error occured while creating an ephemeral messsage {err}"),
    }
}
