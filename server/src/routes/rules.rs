use askama::Template;
use axum::{response::Redirect, Extension, Form};
use chrono::NaiveDate;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::database::entities::{category, rule};

#[derive(Template)]
#[template(path = "rules.html")]
struct RulesTemplate {
    rules: Vec<RuleForTemplate>,
    categories: Vec<category::Model>,
}

struct RuleForTemplate {
    name: String,
    label: String,
    percentage: f32,
    category_id: i32,
    regexpr: String,
    date_start: String,
    date_end: String,
}

#[derive(serde::Deserialize)]
pub struct AddRuleForm {
    name: String,
    label: String,
    percentage: f32,
    category_id: i32,
    regexpr: Option<String>,
    date_start: Option<String>,
    date_end: Option<String>,
}

pub async fn get_rules_handler(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let rules = rule::Entity::find()
        .all(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let categories = category::Entity::find()
        .all(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let rules_for_template: Vec<RuleForTemplate> = rules
        .into_iter()
        .map(|r| RuleForTemplate {
            name: r.name,
            label: r.label,
            percentage: r.percentage,
            category_id: r.category_id,
            regexpr: r.regexpr.unwrap_or_default(),
            date_start: r.date_start.map(|d| d.to_string()).unwrap_or_default(),
            date_end: r.date_end.map(|d| d.to_string()).unwrap_or_default(),
        })
        .collect();

    let html = RulesTemplate {
        rules: rules_for_template,
        categories,
    };

    Ok(axum::response::Html(html.render().unwrap()))
}

pub async fn add_rule_handler(
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddRuleForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    // parse facoltativo della data di inizio
    let date_start = match &form.date_start {
        Some(s) if !s.is_empty() => Some(
            NaiveDate::parse_from_str(s, "%Y-%m-%dT%H:%M")
                .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?,
        ),
        _ => None,
    };

    // parse facoltativo della data di fine
    let date_end = match &form.date_end {
        Some(s) if !s.is_empty() => Some(
            NaiveDate::parse_from_str(s, "%Y-%m-%dT%H:%M")
                .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?,
        ),
        _ => None,
    };

    let new_rule = rule::ActiveModel {
        name: Set(form.name),
        label: Set(form.label),
        percentage: Set(form.percentage),
        category_id: Set(form.category_id),
        regexpr: Set(form.regexpr.clone()), // già Option<String>
        date_start: Set(date_start),
        date_end: Set(date_end),
        ..Default::default()
    };

    new_rule.insert(&db).await.map_err(|e| {
        eprintln!("Error inserting rule: {:?}", e);
        axum::http::StatusCode::BAD_REQUEST
    })?;

    Ok(Redirect::to("/rules"))
}
