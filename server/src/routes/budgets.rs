use askama::Template;
use axum::{response::Redirect, Extension, Form};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::database::entities::{account, budget};

#[derive(askama::Template)]
#[template(path = "budgets.html")]
struct BudgetsTemplate {
    budgets: Vec<budget::Model>,
    accounts: Vec<account::Model>,
}

#[derive(serde::Deserialize)]
pub struct AddBudgetForm {
    name: String,
    value: f64,
    account_id: i32,
}

pub async fn get_budgets_handler(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let budgets = budget::Entity::find().all(&db).await.map_err(|e| {
        eprintln!("Errore find budgets: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let accounts = account::Entity::find().all(&db).await.map_err(|e| {
        eprintln!("Errore find accounts: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let html = BudgetsTemplate { budgets, accounts };
    Ok(axum::response::Html(html.render().unwrap()))
}

pub async fn add_budget_handler(
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddBudgetForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    let new_budget = budget::ActiveModel {
        name: Set(form.name),
        value: Set(form.value),
        account_id: Set(form.account_id),
        ..Default::default()
    };

    new_budget.insert(&db).await.map_err(|e| {
        eprintln!("Errore inserimento budget: {:?}", e);
        axum::http::StatusCode::BAD_REQUEST
    })?;

    Ok(Redirect::to("/budgets"))
}
