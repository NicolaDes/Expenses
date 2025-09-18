use axum::{extract::Path, response::IntoResponse, Extension};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::database::transaction;

pub async fn delete_transaction(
    Path(transaction_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match transaction::Entity::delete_by_id(transaction_id)
        .exec(&db)
        .await
    {
        Ok(_) => axum::http::StatusCode::NO_CONTENT,
        Err(err) => {
            eprintln!("Errore eliminando transazione {}: {}", transaction_id, err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
