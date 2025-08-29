use axum::{Extension, Router};

use axum::serve;
use sea_orm::Database;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod database;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")?;
    let db = Database::connect(&database_url).await?;

    let app = Router::new()
        .nest("/accounts", routes::accounts::router())
        .layer(Extension(db));

    let addr = SocketAddr::from(([0, 0, 0, 0], 7000));
    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();

    Ok(())
}
