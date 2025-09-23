use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Form};
use chrono::NaiveDate;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use serde::Deserialize;

use crate::database::transaction;

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

pub async fn edit_transaction(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<TransactionForm>,
) -> impl IntoResponse {
    let mut transaction: transaction::ActiveModel = transaction::Entity::find_by_id(account_id)
        .one(&db)
        .await
        .expect("Error reading the account!")
        .unwrap()
        .into();

    transaction.account_id = Set(form.account_id);
    transaction.category_id = Set(form.category_id);
    transaction.value = Set(form.value);
    transaction.description = Set(form.description);

    match NaiveDate::parse_from_str(&form.date, "%Y-%m-%dT%H:%M") {
        Ok(date) => transaction.date = Set(date.into()),
        Err(e) => {
            eprintln!("Errore parsing data '{}': {}", form.date, e);
            return StatusCode::BAD_REQUEST;
        }
    }

    transaction.perc_to_exclude = Set(form.perc_to_exclude);
    transaction.label = Set(form.label);

    let _ = transaction.update(&db).await.map_err(|err| {
        eprintln!("Cannot update transaction: {}", err);
        return StatusCode::INTERNAL_SERVER_ERROR;
    });

    return StatusCode::OK;
}
