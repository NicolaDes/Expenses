use crate::database::{
    category,
    entities::{account, transaction},
};
use askama::Template;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{Html, Redirect},
    Form,
};
use chrono::NaiveDateTime;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::Deserialize;

#[derive(Debug)]
struct TransactionWithCategory {
    txt: transaction::Model,
    category_name: String,
}

#[derive(Template)]
#[template(path = "account_transactions.html")]
struct AccountTransactionsTemplate<'a> {
    account: account::Model,
    transactions: Vec<TransactionWithCategory>,
    categories: Vec<category::Model>,
    menu: &'a str,
    sub_menu: &'a str,
}

#[derive(serde::Deserialize)]
pub struct AddTransactionForm {
    description: String,
    value: f64,
    perc_to_exclude: f32,
    label: String,
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
    let account_data = account::Entity::find_by_id(account_id)
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore nel recupero account: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let txs_with_cats = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .find_with_related(category::Entity)
        .all(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore find_with_related: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let transactions: Vec<TransactionWithCategory> = txs_with_cats
        .into_iter()
        .map(|(txt, cats)| {
            let category_name = cats
                .into_iter()
                .next()
                .map(|c| c.category)
                .unwrap_or_else(|| "-".to_string());

            TransactionWithCategory { txt, category_name }
        })
        .collect();

    let categories = match category::Entity::find().all(&db).await {
        Ok(cats) => cats,
        Err(e) => {
            println!("Errore find categories: {:?}", e);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let html = AccountTransactionsTemplate {
        account: account_data,
        transactions,
        categories,
        menu: "accounts",
        sub_menu: "transactions",
    };

    Ok(Html(html.render().unwrap()))
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
        category_id: Set(form.category_id),
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
