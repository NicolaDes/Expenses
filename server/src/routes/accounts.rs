use askama::Template;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect};
use axum::Form;
use axum::{response::Html, Extension};
use sea_orm::DatabaseConnection;

use crate::database::accounts;
use crate::database::AccountModel;

#[derive(Template)]
#[template(path = "accounts.html")]
struct AccountsTemplate<'a> {
    accounts: &'a [AccountModel],
    menu: &'a str,
}

#[derive(serde::Deserialize)]
pub struct NewAccountForm {
    name: String,
}

pub async fn get_all_accounts_handler(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let accounts = accounts::get_accounts(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let html = AccountsTemplate {
        accounts: &accounts,
        menu: "accounts",
    }
    .render()
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Html(html))
}

pub async fn create_account(
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<NewAccountForm>,
) -> impl IntoResponse {
    match accounts::create_account(&db, form.name).await {
        Ok(_) => Redirect::to("/accounts").into_response(),
        Err(err) => Html(err.to_string()).into_response(),
    }
}

pub async fn delete_account(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match accounts::delete_account(&db, account_id).await {
        Ok(_) => (axum::http::StatusCode::NO_CONTENT).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}
