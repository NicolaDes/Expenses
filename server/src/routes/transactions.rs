use axum::{extract::Path, response::IntoResponse, Extension, Form};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::database::transactions;

fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    if let Some(s) = opt {
        if s.trim().is_empty() {
            Ok(None)
        } else {
            s.parse::<i32>().map(Some).map_err(serde::de::Error::custom)
        }
    } else {
        Ok(None)
    }
}

#[derive(Deserialize)]
pub struct TransactionForm {
    account_id: i32,
    #[serde(deserialize_with = "empty_string_as_none")]
    category_id: Option<i32>,
    value: f64,
    description: String,
    date: String,
    perc_to_exclude: f32,
    #[serde(deserialize_with = "empty_string_as_none")]
    label_id: Option<i32>,
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
        form.label_id,
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
