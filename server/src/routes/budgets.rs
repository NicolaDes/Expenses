use askama::Template;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Form};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::database::{account, budget, budgets};

#[derive(Template)]
#[template(path = "budgets.html")]
pub struct BudgetsTemplate<'a> {
    budgets: Vec<BudgetWithCategory>,
    accounts: Vec<account::Model>,
    menu: &'a str,
}

#[derive(Debug)]
struct BudgetWithCategory {
    model: budget::Model,
    account_name: String,
}

#[derive(Deserialize)]
pub struct BudgetForm {
    account_id: i32,
    name: String,
    value: f64,
}

pub async fn get_budgets_handler(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    use sea_orm::EntityTrait;

    let budgets_with_cats = budget::Entity::find()
        .find_with_related(account::Entity)
        .all(&db)
        .await
        .map_err(|err| {
            eprint!("Error find_with_related: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let budgets_list = budgets_with_cats
        .into_iter()
        .map(|(bud, acc)| {
            let account_name = acc
                .into_iter()
                .next()
                .map(|a| a.name)
                .unwrap_or_else(|| "N/A".to_string());

            BudgetWithCategory {
                model: bud,
                account_name,
            }
        })
        .collect();

    let accounts = account::Entity::find().all(&db).await.map_err(|err| {
        eprintln!("Error finding accounts: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let html = BudgetsTemplate {
        budgets: budgets_list,
        accounts,
        menu: "budgets",
    };
    let rendered = html.render().map_err(|e| {
        eprintln!("Template render error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(axum::response::Html(rendered))
}

pub async fn delete_budget(
    Path(budget_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match budgets::delete_budget(&db, budget_id).await {
        Ok(_) => axum::http::StatusCode::NO_CONTENT,
        Err(err) => {
            eprintln!("Error deleting budget {}: {}", budget_id, err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn edit_budget(
    Path(budget_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<BudgetForm>,
) -> impl IntoResponse {
    match budgets::edit_budget(&db, budget_id, form.account_id, form.name, form.value).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            eprintln!("Cannot update budget {}: {}", budget_id, err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
