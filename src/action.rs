use serde::Deserialize;
use sqlx::{postgres::PgQueryResult, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Copy, Deserialize, sqlx::Type)]
#[sqlx(type_name = "action_type", rename_all = "lowercase")]
pub enum ActionType {
    #[default]
    CreateModal = 0,
}

#[derive(Debug, Clone, Default, Deserialize, sqlx::FromRow)]
pub struct Action {
    pub id: Uuid,
    pub slack_id: String,
    pub slack_user: String,
    pub slack_channel: String,
    pub r#type: ActionType,
}

impl Action {
    pub fn new(
        r#type: ActionType,
        slack_id: String,
        slack_user: String,
        slack_channel: String,
    ) -> Self {
        Self {
            r#type,
            slack_id,
            slack_user,
            slack_channel,
            ..Default::default()
        }
    }

    pub fn assign_id(&mut self) -> &mut Self {
        self.id = Uuid::new_v4();
        self
    }

    pub async fn insert(&self, db: &PgPool) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query(r#"INSERT INTO actions (id, "type", slack_id, slack_user, slack_channel) VALUES ($1, $2, $3, $4, $5)"#)
        .bind(self.id).bind(self.r#type).bind(self.slack_id.clone()).bind(self.slack_user.clone()).bind(self.slack_channel.clone())
        .execute(db).await
    }

    pub async fn delete(&self, db: &PgPool) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query(r#"DELETE FROM actions WHERE id=$1"#)
            .bind(self.id)
            .execute(db)
            .await
    }

    pub async fn fetch_slack_id(slack_id: String, db: &PgPool) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<sqlx::Postgres, Action>(
            r#"SELECT * FROM actions WHERE slack_id = $1 LIMIT 1"#,
        )
        .bind(slack_id)
        .fetch_one(db)
        .await
    }
}
