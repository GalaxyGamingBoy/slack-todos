use std::env;

use dotenvy::dotenv;
use sqlx::{migrate, postgres::PgPoolOptions};

mod router;
mod slack;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().unwrap();

    // Connect to database
    print!("Connecting to postgres... ");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await?;
    println!("OK!");

    // Run database migrations
    print!("Running migrations... ");
    migrate!("./migrations").run(&pool).await.unwrap();
    println!("OK!");

    // Run slack app
    let slack = slack::SlackApp::new();
    slack
        .send_message(
            "Hello! Glad to be back online".into(),
            env::var("SLACK_LOG_CHANNEL").unwrap(),
        )
        .await
        .unwrap();

    // Run axum server
    let server = format!("{}:{}", env::var("IP").unwrap(), env::var("PORT").unwrap());

    println!("Running Axum server on: {}", server);
    let listener = tokio::net::TcpListener::bind(&server).await.unwrap();
    axum::serve(listener, router::get_router()).await.unwrap();

    Ok(())
}
