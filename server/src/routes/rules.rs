use askama::Template;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Form};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::database::{categories, category, rules};

#[derive(Template)]
#[template(path = "rules.html")]
struct RulesTemplate<'a> {
    rules: Vec<RuleWithCategory>,
    categories: Vec<category::Model>,
    menu: &'a str,
}

#[derive(Debug)]
struct RuleWithCategory {
    id: i32,
    name: String,
    label: String,
    percentage: f32,
    category_id: Option<i32>,
    category_name: String,
    regexpr: String,
    date_start: String,
    date_end: String,
}

#[derive(Deserialize)]
pub struct RuleForm {
    name: String,
    label: String,
    percentage: f32,
    category_id: i32,
    regexpr: String,
    date_start: String,
    date_end: String,
}

pub async fn get_rules_handler(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let rules_with_cats = rules::get_rules_with_categories(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rules: Vec<RuleWithCategory> = rules_with_cats
        .into_iter()
        .map(|(model, cats)| {
            let cat = cats.into_iter().next();
            let category_id = cat.clone().map(|c| c.id).unwrap();
            let category_name = cat.map(|c| c.category).unwrap_or_else(|| "-".to_string());

            RuleWithCategory {
                id: model.id,
                name: model.name,
                label: model.label,
                percentage: model.percentage,
                category_id: Some(category_id),
                category_name,
                regexpr: model.regexpr.unwrap_or("".to_string()),
                date_start: model.date_start.map(|d| d.to_string()).unwrap_or_default(),
                date_end: model.date_end.map(|d| d.to_string()).unwrap_or_default(),
            }
        })
        .collect();

    let categories = categories::get_categories(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let html = RulesTemplate {
        rules,
        categories,
        menu: "rules",
    };
    Ok(axum::response::Html(
        html.render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn delete_rule(
    Path(rule_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match rules::delete_rule(&db, rule_id).await {
        Ok(_) => (axum::http::StatusCode::NO_CONTENT).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

pub async fn edit_rule(
    Path(rule_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<RuleForm>,
) -> impl IntoResponse {
    match rules::edit_rule(
        &db,
        rule_id,
        form.name,
        form.label,
        form.percentage,
        form.category_id,
        Some(form.regexpr),
        form.date_start,
        form.date_end,
    )
    .await
    {
        Ok(_) => (axum::http::StatusCode::OK).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}
