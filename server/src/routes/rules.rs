use askama::Template;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Form};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::database::{categories, category, entities::label, labels, rules};

#[derive(Template)]
#[template(path = "rules.html")]
struct RulesTemplate<'a> {
    rules: Vec<RuleWithCategory>,
    categories: Vec<category::Model>,
    labels: Vec<label::Model>,
    menu: &'a str,
}

#[derive(Debug)]
struct RuleWithCategory {
    id: i32,
    name: String,
    label_name: String,
    label_id: i32,
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
    label_id: i32,
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

    let all_labels = labels::get_all_labels(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let label_map: std::collections::HashMap<i32, String> = all_labels
        .iter()
        .map(|l| (l.id, l.name.clone()))
        .collect();

    let rules: Vec<RuleWithCategory> = rules_with_cats
        .into_iter()
        .map(|(model, cats)| {
            let cat = cats.into_iter().next();
            let category_id = cat.as_ref().map(|c| c.id);
            let category_name = cat.map(|c| c.category).unwrap_or_else(|| "-".to_string());
            let label_name = label_map
                .get(&model.label_id)
                .cloned()
                .unwrap_or_default();
            let label_id = model.label_id;

            RuleWithCategory {
                id: model.id,
                name: model.name,
                label_name,
                label_id,
                percentage: model.percentage,
                category_id,
                category_name,
                regexpr: model.regexpr.unwrap_or_default(),
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
        labels: all_labels,
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
        form.label_id,
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
