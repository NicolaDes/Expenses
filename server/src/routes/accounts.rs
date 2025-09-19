use askama::Template;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect};
use axum::Form;
use axum::{response::Html, Extension};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::database::account::{self, ActiveModel};
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
    let accounts = accounts::get_all_accounts(&db)
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
    let new_account = ActiveModel {
        name: Set(form.name),
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

pub async fn delete_account(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match account::Entity::delete_by_id(account_id).exec(&db).await {
        Ok(_) => axum::http::StatusCode::NO_CONTENT,
        Err(err) => {
            eprintln!("Errore eliminando transazione {}: {}", account_id, err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
