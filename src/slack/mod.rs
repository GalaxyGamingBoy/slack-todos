use std::env;

use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Response,
};

use serde_json::{json, Value};

use self::modal::SlackModal;

#[derive(Debug, Default, Clone)]
pub struct SlackApp {
    client: reqwest::Client,
}

impl SlackApp {
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            "application/json; charset=utf-8".parse().unwrap(), // Unwrap kept; Hardcoded data
        );
        headers.insert(
            AUTHORIZATION,
            format!(
                "Bearer {}",
                env::var("SLACK_TOKEN")
                    .expect("Can't find SLACK_TOKEN environment variable, is it there?")
            )
            .parse()
            .expect("Bearer token is not valid! Is the SLACK_TOKEN environment variable correct?"),
        );

        Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                // .danger_accept_invalid_certs(true)
                // .proxy(reqwest::Proxy::https("http://localhost:8080").unwrap())
                .build()
                .expect("An error occured while building the reqwest client!"),
        }
    }

    async fn validate_slack(&self, response: Response) -> Result<Value, Value> {
        let data = response.json().await;

        let data: Value = match data {
            Ok(v) => v,
            Err(err) => {
                println!("An error occured while deserializing the slack data! {err}");
                return Err(json!({"msg": "Deserialization error"}));
            }
        };

        if data["ok"].as_bool().unwrap_or(false) == false {
            println!("Slack API Error! {:?}", data);
            return Err(data);
        }

        Ok(data)
    }

    pub async fn send_message(&self, text: String, channel: String) -> Result<Value, Value> {
        match self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .json(&json!({"text": text, "channel": channel}))
            .send()
            .await
        {
            Ok(v) => self.validate_slack(v).await,
            Err(err) => {
                println!("An error occured while sending request to slack API: {err}");
                Err(json!({"msg": "Slack API Request Error"}))
            }
        }
    }

    pub async fn send_block(&self, channel: String, block: &mut Value) {
        let mut data: Value = Value::default();
        data["blocks"] = block.clone();
        data["channel"] = Value::String(channel);

        match self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .json(&data)
            .send()
            .await
        {
            Ok(_) => {}
            Err(err) => println!("An error occured while sending a slack webhook: {err}"),
        }
    }

    pub async fn send_ephemeral(
        &self,
        blocks: String,
        channel: String,
        user: String,
    ) -> Result<Value, Value> {
        let body = format!(
            r#"{{"blocks": {}, "channel": "{}", "user": "{}"}}"#,
            blocks, channel, user
        );

        match self
            .client
            .post("https://slack.com/api/chat.postEphemeral")
            .body(body)
            .send()
            .await
        {
            Ok(v) => self.validate_slack(v).await,
            Err(err) => {
                println!("An error occured while sending request to slack API: {err}");
                Err(json!({"msg": "Slack API Request Error"}))
            }
        }
    }

    pub async fn send_webhook(&self, webhook: String, block: &mut Value, ephemeral: bool) {
        let data = block;
        if ephemeral {
            data["response_type"] = Value::String("ephemeral".to_string())
        }

        match self.client.post(webhook).json(data).send().await {
            Ok(_) => {}
            Err(err) => println!("An error occured while sending a slack webhook: {err}"),
        }
    }

    pub async fn open_modal(&self, modal: &SlackModal) -> Result<Value, Value> {
        let data = format!(
            r#"{{"trigger_id": "{}", "view": {}}}"#,
            modal.trigger, modal.data
        );

        match self
            .client
            .post("https://slack.com/api/views.open")
            .body(data)
            .send()
            .await
        {
            Ok(v) => self.validate_slack(v).await,
            Err(err) => {
                println!("An error occured while sending request to slack API: {err}");
                Err(json!({"msg": "Slack API Request Error"}))
            }
        }
    }
}

pub mod block;
pub mod escape;
pub mod modal;
pub mod payloads;
