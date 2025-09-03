use crate::database::entities::{account, category, transaction};
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
    categories: Vec<category::Model>,
}

#[derive(serde::Deserialize)]
pub struct AddTransactionForm {
    description: String,
    value: f64,
    perc_to_exclude: f32,
    label: String,
    date: String,
    category_id: Option<i32>,
}

pub async fn get_account_transactions_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let account_data = match account::Entity::find_by_id(account_id).one(&db).await {
        Ok(Some(account)) => account,
        Ok(None) => return Err(axum::http::StatusCode::NOT_FOUND),
        Err(e) => {
            println!("Errore find_by_id: {:?}", e);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let transactions = match account_data
        .find_related(transaction::Entity)
        .all(&db)
        .await
    {
        Ok(txs) => txs,
        Err(e) => {
            println!("Errore find_related: {:?}", e);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let categories = match category::Entity::find().all(&db).await {
        Ok(cats) => cats,
        Err(e) => {
            println!("Errore find categories: {:?}", e);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let html = TransactionsTemplate {
        transactions,
        account_name: account_data.name,
        account_id,
        categories,
    };
    Ok(axum::response::Html(html.render().unwrap()))
}

pub async fn add_transaction_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddTransactionForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    let naive_date = NaiveDateTime::parse_from_str(&form.date, "%Y-%m-%dT%H:%M")
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    let new_tx = transaction::ActiveModel {
        account_id: Set(account_id),
        category_id: Set(form.category_id), // Ora puoi impostare la categoria
        description: Set(form.description),
        value: Set(form.value),
        perc_to_exclude: Set(form.perc_to_exclude),
        label: Set(form.label),
        date: Set(naive_date),
        ..Default::default()
    };
    if let Err(e) = new_tx.insert(&db).await {
        eprintln!("Errore inserimento transaction: {:?}", e);
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }
    Ok(Redirect::to(&format!(
        "/accounts/{}/transactions",
        account_id
    )))
}
