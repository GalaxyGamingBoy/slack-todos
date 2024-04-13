use serde::{Deserialize, Serialize};
use serde_json::Value;

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
