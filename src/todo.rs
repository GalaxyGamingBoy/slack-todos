use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, PgPool};

use crate::slack::block::SlackBlock;

#[derive(Debug, Serialize, Deserialize, Default, sqlx::FromRow)]
pub struct Todo {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub slack_user: String,
}

impl Todo {
    pub fn assign_id(&mut self) -> &mut Self {
        self.id = uuid::Uuid::new_v4();

        self
    }

    pub async fn insert(&self, db: &PgPool) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(r#"INSERT INTO todos (id, title, description, completed, slack_user) VALUES ($1, $2, $3, $4, $5)"#,
            self.id, self.title, self.description, self.completed, self.slack_user).execute(db).await
    }

    pub fn block(&mut self) -> SlackBlock {
        let mut template: HashMap<&str, String> = HashMap::new();
        template.insert("id", self.id.to_string());
        template.insert("title", self.title.clone());
        template.insert(
            "desc",
            self.description
                .clone()
                .unwrap_or("_No Description_".to_string()),
        );

        let mut block = SlackBlock::new("todo".to_string());
        block.load().fill(template);

        block
    }
}
