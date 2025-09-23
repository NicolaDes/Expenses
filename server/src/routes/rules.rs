use askama::Template;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Form};
use chrono::NaiveDate;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use serde::Deserialize;

use crate::database::{category, entities::rule};

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
    let rules_with_cats = rule::Entity::find()
        .find_with_related(category::Entity)
        .all(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore find_with_related: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

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
                category_name: category_name,
                regexpr: model.regexpr.unwrap_or("".to_string()),
                date_start: model.date_start.map(|d| d.to_string()).unwrap_or_default(),
                date_end: model.date_end.map(|d| d.to_string()).unwrap_or_default(),
            }
        })
        .collect();

    let categories = category::Entity::find().all(&db).await.map_err(|err| {
        eprintln!("Error finding categories: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let html = RulesTemplate {
        rules,
        categories,
        menu: "rules",
    };
    Ok(axum::response::Html(html.render().unwrap()))
}

pub async fn delete_rule(
    Path(rule_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match rule::Entity::delete_by_id(rule_id).exec(&db).await {
        Ok(_) => axum::http::StatusCode::NO_CONTENT,
        Err(err) => {
            eprintln!("Errore eliminando transazione {}: {}", rule_id, err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn edit_rule(
    Path(rule_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<RuleForm>,
) -> impl IntoResponse {
    let mut rule: rule::ActiveModel = rule::Entity::find_by_id(rule_id)
        .one(&db)
        .await
        .expect("Error reading the rule!")
        .unwrap()
        .into();

    rule.name = Set(form.name);
    rule.label = Set(form.label);
    rule.percentage = Set(form.percentage);
    rule.category_id = Set(form.category_id);
    rule.regexpr = Set(Some(form.regexpr));
    rule.date_start = Set(if form.date_start.trim().is_empty() {
        None
    } else {
        match NaiveDate::parse_from_str(&form.date_start, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => return StatusCode::BAD_REQUEST,
        }
    });

    rule.date_end = Set(if form.date_end.trim().is_empty() {
        None
    } else {
        match NaiveDate::parse_from_str(&form.date_end, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => return StatusCode::BAD_REQUEST,
        }
    });

    let _ = rule.update(&db).await.map_err(|err| {
        eprintln!("Cannot update rule: {}", err);
        return StatusCode::INTERNAL_SERVER_ERROR;
    });

    return StatusCode::OK;
}
