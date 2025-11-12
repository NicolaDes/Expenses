use askama::Template;
use axum::{extract::Path, http::StatusCode, response::Redirect, Extension, Form};
use sea_orm::DatabaseConnection;

use crate::database::{
    accounts, budgets,
    entities::{account, budget},
};

#[derive(Template)]
#[template(path = "account_budgets.html")]
struct BudgetsTemplate<'a> {
    account: account::Model,
    budgets: Vec<budget::Model>,
    menu: &'a str,
    sub_menu: &'a str,
}

#[derive(serde::Deserialize)]
pub struct AddBudgetForm {
    name: String,
    value: f64,
}

pub async fn get_account_budgets_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let account_data = accounts::get_account(&db, account_id).await.map_err(|e| {
        eprintln!("DB error get_account_by_id: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let budgets = budgets::get_budgets_for_account(&db, account_id)
        .await
        .map_err(|e| {
            eprintln!("DB error get_budgets_for_account: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let html = BudgetsTemplate {
        account: account_data,
        budgets,
        menu: "accounts",
        sub_menu: "budgets",
    };
    Ok(axum::response::Html(html.render().unwrap()))
}

pub async fn add_budget_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddBudgetForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    match budgets::create_budget(&db, account_id, form.name, form.value).await {
        Ok(_) => Ok(Redirect::to(&format!("/accounts/{}/budgets", account_id))),
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}
