use std::env;

use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Response,
};
use serde_json::{json, Value};

#[derive(Debug, Default, Clone)]
pub struct SlackApp {
    client: reqwest::Client,
}

impl SlackApp {
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            "application/json; charset=utf-8".parse().unwrap(),
        );
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", env::var("SLACK_TOKEN").unwrap())
                .parse()
                .unwrap(),
        );

        Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }

    async fn validate_slack(&self, response: Response) -> Result<Value, Value> {
        let data: Value = response.json().await.unwrap();

        if data["ok"].as_bool().unwrap() == false {
            println!("Slack API Error! {:?}", data);
            return Err(data);
        }

        Ok(data)
    }

    pub async fn send_message(&self, text: String, channel: String) -> Result<Value, Value> {
        let res = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .json(&json!({"text": text, "channel": channel}))
            .send()
            .await
            .unwrap();

        self.validate_slack(res).await
    }
}
