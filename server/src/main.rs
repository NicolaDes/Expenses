use axum::{serve, Extension};
use sea_orm::Database;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod database;
mod routes;
use crate::routes::routes::router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // prendi l'URL del database dalle variabili d'ambiente
    let database_url = std::env::var("DATABASE_URL")?;
    let db = Database::connect(&database_url).await?;

    // usa il router centrale da routes.rs
    let app = router().layer(Extension(db));

    // bind del server
    let addr = SocketAddr::from(([0, 0, 0, 0], 7000));
    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}
