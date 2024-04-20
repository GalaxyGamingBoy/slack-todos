use std::collections::HashMap;

use axum::{
    extract::State,
    routing::{get, post},
    Form, Router,
};
use sqlx::query;

use crate::{
    action::{self, Action, ActionType},
    slack::{
        block::SlackBlock,
        escape::SlackEscape,
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
        .route("/todo/list", post(todo_list))
        .route("/slack/interactivity", post(slack_interactivity))
}

async fn root() -> &'static str {
    "Hello, Slack To-Do!"
}

async fn todo_new(State(state): State<ServerState>, Form(payload): Form<SlackCommand>) {
    if payload.text.trim().is_empty() {
        let mut template: HashMap<&str, String> = HashMap::new();
        template.insert("initial_channel", payload.channel_id.clone());

        let mut modal = SlackModal::new("create".to_string(), payload.trigger_id);
        modal.load().fill(template);

        match state.slack.open_modal(&modal).await {
            Ok(v) => {
                let id = match v["view"]["id"].as_str() {
                    Some(v) => v,
                    None => {
                        println!("ID not found in payload");
                        return;
                    }
                };

                let mut action = Action::new(
                    ActionType::CreateModal,
                    id.to_string(),
                    payload.user_id,
                    payload.channel_id,
                );

                match action.assign_id().insert(&state.db).await {
                    Ok(_) => {}
                    Err(err) => println!("Action to database insertion error! {err}"),
                }
            }
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

async fn todo_list(State(state): State<ServerState>, Form(payload): Form<SlackCommand>) {
    let target = if payload.text.len() == 0 {
        SlackEscape {
            id: payload.user_id.clone(),
            display: payload.user_name.clone(),
        }
    } else {
        SlackEscape::from(payload.text)
    };

    let query = sqlx::query_as::<sqlx::Postgres, Todo>(
        r#"SELECT * FROM todos WHERE slack_user = $1 LIMIT 5"#,
    )
    .bind(target.id)
    .fetch_all(&state.db)
    .await;

    let mut query = match query {
        Ok(v) => v,
        Err(err) => {
            println!("Failed to fetch todos! {err}");
            return;
        }
    };

    if query.len() == 0 {
        match state
            .slack
            .send_message(
                format!("No todos found for <@{}>", target.display),
                payload.channel_id.clone(),
            )
            .await
        {
            Ok(_) => {}
            Err(err) => println!("An error occured while sending a slack message. {err}"),
        };

        return;
    }

    let todos = query
        .iter_mut()
        .map(|todo| todo.block().data)
        .collect::<Vec<String>>()
        .join(",");

    let mut template: HashMap<&str, String> = HashMap::new();
    template.insert("list", todos);
    template.insert("user", target.display);

    let mut block = SlackBlock::new("list".to_string());
    block.load().fill(template).trim();

    state
        .slack
        .send_block(payload.channel_id, &mut block.into())
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

    if payload.r#type == "view_submission" {
        let action = Action::fetch_slack_id(payload.view.id.clone(), &state.db)
            .await
            .unwrap();

        match action.r#type {
            ActionType::CreateModal => create_modal(&payload, &state, &action).await,
            _ => unreachable!(),
        }
    };
}

async fn create_modal(payload: &SlackInteractionData, state: &ServerState, action: &Action) {
    action.delete(&state.db).await.unwrap();

    let title =
        &payload.view.state["values"]["input-title"]["input-title-action"]["value"].as_str();
    let description = &payload.view.state["values"]["input-description"]
        ["input-description-action"]["value"]
        .as_str();

    let mut todo = Todo {
        title: title
            .expect("Slack Interaction did not contain title! Is it in the modal?")
            .to_string(),
        description: description.map(str::to_string),
        slack_user: action.slack_user.clone(),
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

    match state
        .slack
        .send_ephemeral(
            block.data,
            action.slack_channel.clone(),
            action.slack_user.clone(),
        )
        .await
    {
        Ok(_) => {}
        Err(err) => println!("An error occured while creating an ephemeral messsage {err}"),
    }
}
