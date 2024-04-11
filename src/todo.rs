use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize, Default)]
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

    pub async fn insert(&self, db: &PgPool) -> &Self {
        sqlx::query!(r#"INSERT INTO todos (id, title, description, completed, slack_user) VALUES ($1, $2, $3, $4, $5)"#,
            self.id, self.title, self.description, self.completed, self.slack_user).execute(db).await.unwrap();

        self
    }
}
