use std::env;

use dotenvy::dotenv;
use slack::SlackApp;
use sqlx::{migrate, postgres::PgPoolOptions, types::chrono, PgPool};

mod action;
mod router;
mod slack;
mod todo;

#[derive(Clone)]
pub struct ServerState {
    db: PgPool,
    slack: SlackApp,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match dotenv() {
        Ok(_) => println!("Loaded .env file"),
        Err(_) => println!(".env file not found, ignoring..."),
    }

    // Connect to database
    print!("Connecting to postgres... ");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            &env::var("DATABASE_URL").expect("PG Database URL not found in environment variables"),
        )
        .await?;
    println!("OK!");

    // Run database migrations
    print!("Running migrations... ");
    migrate!("./migrations")
        .run(&pool)
        .await
        .expect("An error occured while running migrations");
    println!("OK!");

    // Run slack app
    let slack = slack::SlackApp::new();
    let res = slack
        .send_message(
            format!(
                "[START]: {} - v{} ({})",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                chrono::Utc::now().to_rfc3339()
            ),
            env::var("SLACK_LOG_CHANNEL")
                .expect("SLACK_LOG_CHANNEL environment vairable not found"),
        )
        .await;

    match res {
        Ok(_) => {}
        Err(err) => println!("An error occured while sending start message: {err}"),
    }

    // Run axum server
    let server = format!(
        "{}:{}",
        env::var("IP").unwrap_or("0.0.0.0".to_string()),
        env::var("PORT").unwrap_or("3000".to_string())
    );

    println!("Running Axum server on: {}", server);
    let listener = tokio::net::TcpListener::bind(&server)
        .await
        .expect("An error occured while creating TCP Listener");
    axum::serve(
        listener,
        router::get_router().with_state(ServerState { db: pool, slack }),
    )
    .await
    .expect("An error occured while running axum server");

    Ok(())
}
