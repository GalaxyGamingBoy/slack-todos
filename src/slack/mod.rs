use std::{collections::HashMap, env, fs};

use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Response,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlackCommand {
    pub team_id: String,
    pub team_domain: String,
    pub channel_id: String,
    pub user_id: String,
    pub user_name: String,
    pub command: String,
    pub text: String,
    pub response_url: String,
    pub trigger_id: String,
    pub api_app_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SlackInteractionUser {
    pub username: String,
    pub name: String,
    pub id: String,
    pub team_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SlackInteractionTeam {
    pub domain: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SlackInteractionView {
    pub id: String,
    pub r#type: String,
    pub team_id: String,
    pub private_metadata: String,
    pub callback_id: String,
    pub state: Value,
    pub blocks: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SlackInteractionData {
    pub r#type: String,
    pub user: SlackInteractionUser,
    pub team: SlackInteractionTeam,
    pub api_app_id: String,
    pub trigger_id: String,
    pub token: String,
    pub view: SlackInteractionView,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SlackInteraction {
    pub payload: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SlackBlock {
    name: String,
    pub data: String,
}

impl SlackBlock {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn load(&mut self) -> &mut Self {
        self.data = fs::read_to_string(format!("./src/blocks/{}.block.json", self.name))
            .expect("Slack block file not found!");
        self
    }

    pub fn fill(&mut self, args: HashMap<&str, String>) -> &mut Self {
        args.iter().for_each(|arg| {
            let key = format!("{{{{{}}}}}", arg.0);

            self.data = self.data.replace(&key, arg.1);
        });

        self
    }

    pub fn trim(&mut self) -> &mut Self {
        let data: Value =
            serde_json::from_str(&self.data).expect("Slack block isn't valid JSON data");
        self.data = data["blocks"].to_string();

        self
    }
}

impl Into<Value> for SlackBlock {
    fn into(self) -> Value {
        serde_json::from_str(&self.data).unwrap_or_default()
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct SlackModal {
    name: String,
    pub data: String,
    pub trigger: String,
}

impl SlackModal {
    pub fn new(name: String, trigger: String) -> Self {
        Self {
            name,
            trigger,
            ..Default::default()
        }
    }

    pub fn load(&mut self) -> &mut Self {
        self.data = fs::read_to_string(format!("./src/modals/{}.modal.json", self.name))
            .expect("Slack modal file wasn't found!");
        self
    }

    pub fn fill(&mut self, args: HashMap<&str, String>) -> &mut Self {
        args.iter().for_each(|arg| {
            let key = format!("{{{{{}}}}}", arg.0);

            self.data = self.data.replace(&key, arg.1);
        });

        self
    }
}

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
