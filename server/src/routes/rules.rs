use askama::Template;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension};
use sea_orm::{DatabaseConnection, EntityTrait};

use crate::database::{category, entities::rule};

#[derive(Template)]
#[template(path = "rules.html")]
struct RulesTemplate<'a> {
    rules: Vec<RuleWithCategory>,
    menu: &'a str,
}

#[derive(Debug)]
struct RuleWithCategory {
    id: i32,
    name: String,
    label: String,
    percentage: f32,
    category_name: String,
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
            let category_name = cats
                .into_iter()
                .next()
                .map(|c| c.category)
                .unwrap_or_else(|| "-".to_string());

            RuleWithCategory {
                id: model.id,
                name: model.name,
                label: model.label,
                percentage: model.percentage,
                category_name: category_name,
                regexpr: model.regexpr.unwrap_or("".to_string()),
                date_start: model.date_start.map(|d| d.to_string()).unwrap_or_default(),
                date_end: model.date_end.map(|d| d.to_string()).unwrap_or_default(),
            }
        })
        .collect();

    let html = RulesTemplate {
        rules,
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
