use axum::{extract::Path, response::IntoResponse, Extension, Form};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::database::transactions;

#[derive(Deserialize)]
pub struct TransactionForm {
    account_id: i32,
    category_id: Option<i32>,
    value: f64,
    description: String,
    date: String,
    perc_to_exclude: f32,
    label: String,
}

pub async fn delete_transaction(
    Path(transaction_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match transactions::delete_transaction(&db, transaction_id).await {
        Ok(_) => (axum::http::StatusCode::NO_CONTENT).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

pub async fn edit_transaction(
    Path(id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<TransactionForm>,
) -> impl IntoResponse {
    match transactions::edit_transaction(
        &db,
        id,
        form.account_id,
        form.category_id,
        form.value,
        form.description,
        form.date,
        form.perc_to_exclude,
        form.label,
    )
    .await
    {
        Ok(_) => (axum::http::StatusCode::OK).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}
