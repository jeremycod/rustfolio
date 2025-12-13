
mod db;
mod routes;
mod models;
mod errors;
mod utils;
mod app;
mod services;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = app::create_app(pool);

/*    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(root));*/

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("🚀 Rustfolio backend running at http://{}/", addr);
    axum::serve(listener, app)
        .await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn root() -> &'static str {
    "Rustfolio backend is alive"
}
