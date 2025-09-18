use askama::Template;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::database::{account, budget};

#[derive(Template)]
#[template(path = "budgets.html")]
pub struct BudgetsTemplate<'a> {
    budgets: Vec<BudgetWithCategory>,
    menu: &'a str,
}

#[derive(Debug)]
struct BudgetWithCategory {
    model: budget::Model,
    account_name: String,
}

pub async fn get_budgets_handler(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let budgets_with_cats = budget::Entity::find()
        .find_with_related(account::Entity)
        .all(&db)
        .await
        .map_err(|err| {
            eprint!("Error find_with_related: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let budgets = budgets_with_cats
        .into_iter()
        .map(|(bud, acc)| {
            let account_name = acc
                .into_iter()
                .next()
                .map(|a| a.name)
                .unwrap_or_else(|| "N/A".to_string());

            BudgetWithCategory {
                model: bud,
                account_name: account_name,
            }
        })
        .collect();

    let html = BudgetsTemplate {
        budgets,
        menu: "budgets",
    };
    Ok(axum::response::Html(html.render().unwrap()))
}

pub async fn delete_budget(
    Path(budget_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match budget::Entity::delete_by_id(budget_id).exec(&db).await {
        Ok(_) => axum::http::StatusCode::NO_CONTENT,
        Err(err) => {
            eprintln!("Errore eliminando transazione {}: {}", budget_id, err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
