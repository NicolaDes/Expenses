use crate::database::entities::{account, transaction};
use askama::Template;
use axum::{extract::Path, response::Redirect, Extension, Form};
use chrono::NaiveDateTime;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, Set};

#[derive(askama::Template)]
#[template(path = "transactions.html")]
struct TransactionsTemplate {
    account_name: String,
    account_id: i32,
    transactions: Vec<transaction::Model>,
}

pub async fn get_account_transactions_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<sea_orm::DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    // Prendi l'account
    let account_data = account::Entity::find_by_id(account_id)
        .one(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    // Prendi tutte le transactions di quell'account
    let transactions = account_data
        .find_related(transaction::Entity)
        .all(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let html = TransactionsTemplate {
        transactions,
        account_name: account_data.name,
        account_id,
    };

    Ok(axum::response::Html(html.render().unwrap()))
}

#[derive(serde::Deserialize)]
pub struct AddTransactionForm {
    description: String,
    value: f64,
    perc_to_exclude: f32,
    label: String,
    date: String,
}

pub async fn add_transaction_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>, // Extension DEVE venire prima di Form
    Form(form): Form<AddTransactionForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    // Parse della data inviata dal form in NaiveDateTime
    let naive_date = NaiveDateTime::parse_from_str(&form.date, "%Y-%m-%dT%H:%M")
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    // Creazione del nuovo record
    let new_tx = transaction::ActiveModel {
        account_id: Set(account_id),
        description: Set(form.description),
        value: Set(form.value),
        perc_to_exclude: Set(form.perc_to_exclude),
        label: Set(form.label),
        date: Set(naive_date), // usa direttamente NaiveDateTime
        ..Default::default()
    };

    // Inserimento nel DB
    new_tx
        .insert(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Redirect alla stessa pagina delle transazioni
    Ok(Redirect::to(&format!(
        "/accounts/{}/transactions",
        account_id
    )))
}
