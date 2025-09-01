use askama::Template;
use axum::response::{IntoResponse, Redirect};
use axum::Form;
use axum::{response::Html, Extension};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

use crate::database::account::ActiveModel;
use crate::database::accounts;
use crate::database::AccountModel;

#[derive(Template)]
#[template(path = "accounts.html")]
struct AccountsTemplate<'a> {
    accounts: &'a [AccountModel],
}

#[derive(Template)]
#[template(path = "new_account.html")]
struct NewAccountTemplate;

#[derive(serde::Deserialize)]
pub struct NewAccountForm {
    name: String,
    balance: f64,
}

pub async fn get_all_accounts_handler(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let accounts = accounts::get_all_accounts(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let html = AccountsTemplate {
        accounts: &accounts,
    }
    .render()
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Html(html))
}

pub async fn new_account_form() -> impl IntoResponse {
    Html(NewAccountTemplate.render().unwrap())
}

pub async fn create_account(
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<NewAccountForm>,
) -> impl IntoResponse {
    let new_account = ActiveModel {
        name: Set(form.name),
        balance: Set(form.balance),
        ..Default::default()
    };

    match new_account.insert(&db).await {
        Ok(_) => Redirect::to("/accounts").into_response(),
        Err(e) => {
            eprintln!("Error inserting account: {}", e);
            Html("Failed to create account".to_string()).into_response()
        }
    }
}
