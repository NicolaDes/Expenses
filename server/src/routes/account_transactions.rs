use crate::database::{
    accounts, categories, labels, transactions,
    entities::{account, label, transaction},
    category,
};
use askama::Template;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{Html, Redirect},
    Form,
};
use chrono::NaiveDateTime;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

#[derive(Debug)]
struct TransactionWithCategory {
    txt: transaction::Model,
    category_name: String,
    label_name: String,
}

#[derive(Template)]
#[template(path = "account_transactions.html")]
struct AccountTransactionsTemplate<'a> {
    account: account::Model,
    transactions: Vec<TransactionWithCategory>,
    categories: Vec<category::Model>,
    labels: Vec<label::Model>,
    menu: &'a str,
    sub_menu: &'a str,
}

#[derive(serde::Deserialize)]
pub struct AddTransactionForm {
    description: String,
    value: f64,
    perc_to_exclude: f32,
    #[serde(deserialize_with = "empty_string_as_none")]
    label_id: Option<i32>,
    date: String,
    #[serde(deserialize_with = "empty_string_as_none")]
    category_id: Option<i32>,
}

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

pub async fn get_account_transactions_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Html<String>, StatusCode> {
    let account_data = accounts::get_account(&db, account_id).await.map_err(|e| {
        eprintln!("Error retrieving account: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let txs_with_cats =
        transactions::get_transactions_with_categories_for_account(&db, account_id)
            .await
            .map_err(|e| {
                eprintln!("Error retrieving transactions with categories: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let all_labels = labels::get_all_labels(&db).await.map_err(|e| {
        eprintln!("Error retrieving labels: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let label_map: std::collections::HashMap<i32, String> = all_labels
        .iter()
        .map(|l| (l.id, l.name.clone()))
        .collect();

    let transaction_list: Vec<TransactionWithCategory> = txs_with_cats
        .into_iter()
        .map(|(txt, cats)| {
            let category_name = cats
                .into_iter()
                .next()
                .map(|c| c.category)
                .unwrap_or_else(|| "-".to_string());
            let label_name = txt
                .label_id
                .and_then(|id| label_map.get(&id))
                .cloned()
                .unwrap_or_default();
            TransactionWithCategory { txt, category_name, label_name }
        })
        .collect();

    let categories = categories::get_categories(&db).await.map_err(|e| {
        eprintln!("Error retrieving categories: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let html = AccountTransactionsTemplate {
        account: account_data,
        transactions: transaction_list,
        categories,
        labels: all_labels,
        menu: "accounts",
        sub_menu: "transactions",
    };

    html.render().map(Html).map_err(|e| {
        eprintln!("Template render error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn add_transaction_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddTransactionForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    let naive_date = NaiveDateTime::parse_from_str(&form.date, "%Y-%m-%dT%H:%M")
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    transactions::create_transaction(
        &db,
        account_id,
        form.category_id,
        form.value,
        form.description,
        naive_date,
        form.perc_to_exclude,
        form.label_id,
    )
    .await
    .map_err(|e| {
        eprintln!("Error inserting transaction: {:?}", e);
        axum::http::StatusCode::BAD_REQUEST
    })?;

    Ok(Redirect::to(&format!(
        "/accounts/{}/transactions",
        account_id
    )))
}
