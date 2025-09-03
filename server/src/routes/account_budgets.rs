use askama::Template;
use axum::{extract::Path, http::StatusCode, response::Redirect, Extension, Form};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, ModelTrait};

use crate::database::entities::{account, budget};

#[derive(Template)]
#[template(path = "account_budgets.html")]
struct BudgetsTemplate<'a> {
    account: account::Model,
    budgets: Vec<budget::Model>,
    // categories: Vec<category::Model>,
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
    let account_data = account::Entity::find_by_id(account_id)
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore nel recupero account: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let budgets = match account_data.find_related(budget::Entity).all(&db).await {
        Ok(txs) => txs,
        Err(e) => {
            println!("Errore find_related: {:?}", e);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // let categories = match category::Entity::find().all(&db).await {
    //     Ok(cats) => cats,
    //     Err(e) => {
    //         println!("Errore find categories: {:?}", e);
    //         return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    //     }
    // };

    let html = BudgetsTemplate {
        account: account_data,
        budgets,
        // categories,
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
    let new_budget = budget::ActiveModel {
        name: Set(form.name),
        value: Set(form.value),
        account_id: Set(account_id),
        ..Default::default()
    };

    new_budget.insert(&db).await.map_err(|e| {
        eprintln!("Errore inserimento budget: {:?}", e);
        axum::http::StatusCode::BAD_REQUEST
    })?;

    Ok(Redirect::to(&format!("/accounts/{}/budgets", account_id)))
}
