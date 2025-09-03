use crate::database::entities::{account, account_rule, rule};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::Html,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use askama::Template;

#[derive(Template)]
#[template(path = "account_rules.html")]
struct AccountRulesTemplate {
    account: account::Model,
    active_rules: Vec<rule::Model>,
    inactive_rules: Vec<rule::Model>,
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

    let html = AccountRulesTemplate {
        account: account_data,
        active_rules,
        inactive_rules,
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
