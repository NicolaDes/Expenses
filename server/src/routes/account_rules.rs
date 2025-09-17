use crate::database::{
    category,
    entities::{account, account_rule, rule},
};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{Html, Redirect},
    Form,
};
use chrono::NaiveDate;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use askama::Template;

#[derive(Template)]
#[template(path = "account_rules.html")]
struct AccountRulesTemplate<'a> {
    account: account::Model,
    active_rules: Vec<rule::Model>,
    inactive_rules: Vec<rule::Model>,
    categories: Vec<category::Model>,
    menu: &'a str,
    sub_menu: &'a str,
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

pub async fn get_account_rules_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Html<String>, StatusCode> {
    let account_data = account::Entity::find_by_id(account_id)
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore nel recupero account: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let active_rules_raw = account::Entity::find_by_id(account_id)
        .find_with_related(rule::Entity)
        .all(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore nel recupero delle regole attive: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let active_rules: Vec<rule::Model> = active_rules_raw
        .into_iter()
        .flat_map(|(_acc, rules)| rules)
        .collect();

    let all_rules = rule::Entity::find().all(&db).await.map_err(|e| {
        eprintln!("Errore nel recupero di tutte le regole: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let inactive_rules: Vec<rule::Model> = all_rules
        .into_iter()
        .filter(|r| !active_rules.iter().any(|ar| ar.id == r.id))
        .collect();

    let categories = category::Entity::find()
        .all(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let html = AccountRulesTemplate {
        account: account_data,
        active_rules,
        inactive_rules,
        categories,
        menu: "accounts",
        sub_menu: "rules",
    };

    Ok(Html(html.render().unwrap()))
}

pub async fn activate_rule_handler(
    Path((account_id, rule_id)): Path<(i32, i32)>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    account_rule::ActiveModel {
        account_id: Set(account_id),
        rule_id: Set(rule_id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

pub async fn deactivate_rule_handler(
    Path((account_id, rule_id)): Path<(i32, i32)>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    account_rule::Entity::delete_many()
        .filter(account_rule::Column::AccountId.eq(account_id))
        .filter(account_rule::Column::RuleId.eq(rule_id))
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

pub async fn add_account_rule_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddRuleForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    let date_start = match &form.date_start {
        Some(s) if !s.is_empty() => Some(
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?,
        ),
        _ => None,
    };

    let date_end = match &form.date_end {
        Some(s) if !s.is_empty() => Some(
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?,
        ),
        _ => None,
    };

    let new_rule = rule::ActiveModel {
        name: Set(form.name),
        label: Set(form.label),
        percentage: Set(form.percentage),
        category_id: Set(form.category_id),
        regexpr: Set(form.regexpr.clone()),
        date_start: Set(date_start),
        date_end: Set(date_end),

        ..Default::default()
    };

    let inserted_rule = new_rule.insert(&db).await.map_err(|e| {
        eprintln!("Error inserting rule: {:?}", e);
        axum::http::StatusCode::BAD_REQUEST
    })?;

    account_rule::ActiveModel {
        account_id: Set(account_id),
        rule_id: Set(inserted_rule.id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to(&format!("/accounts/{}/rules", account_id)))
}
