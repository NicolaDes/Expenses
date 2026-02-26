use std::collections::HashMap;

use crate::database::{
    accounts, categories, rules, transactions,
    category,
    entities::{account, rule, transaction},
};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Form, Json,
};
use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};

use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "account_rules.html")]
struct AccountRulesTemplate<'a> {
    account: account::Model,
    rules: Vec<RuleWithStatus>,
    categories: Vec<category::Model>,
    uncategorized_transactions: Vec<transaction::Model>,
    menu: &'a str,
    sub_menu: &'a str,
}

struct RuleWithStatus {
    model: rule::Model,
    active: bool,
}

#[derive(Serialize)]
pub struct PreviewTransaction {
    id: i32,
    description: String,
    value: f64,
    date: String,
    conflicts: Vec<rule::Model>,
    label_old_value: String,
    label_new_value: String,
    perc_to_exclude_old_value: f32,
    perc_to_exclude_new_value: f32,
    category_old_value: String,
    category_new_value: String,
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

#[derive(Deserialize)]
pub struct ResolveConflictPayload {
    transaction_id: i32,
    rule_id: i32,
}

pub async fn get_account_rules_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Html<String>, StatusCode> {
    let account_data = accounts::get_account(&db, account_id).await.map_err(|e| {
        eprintln!("Error retrieving account data: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let all_rules = rules::get_rules(&db).await.map_err(|e| {
        eprintln!("Error retrieving rules data: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let active_rule_ids = rules::get_active_rule_ids_for_account(&db, account_id)
        .await
        .map_err(|e| {
            eprintln!("Error retrieving active rule ids: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let rules_with_status: Vec<RuleWithStatus> = all_rules
        .into_iter()
        .map(|r| {
            let id = r.id;
            RuleWithStatus {
                model: r,
                active: active_rule_ids.contains(&id),
            }
        })
        .collect();

    let categories_list = categories::get_categories(&db).await.map_err(|e| {
        eprintln!("Error retrieving categories data: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let uncategorized_transactions =
        transactions::get_uncategorized_transactions_for_account(&db, account_id)
            .await
            .map_err(|e| {
                eprintln!("Error retrieving uncategorized transactions: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let html = AccountRulesTemplate {
        account: account_data,
        rules: rules_with_status,
        categories: categories_list,
        uncategorized_transactions,
        menu: "accounts",
        sub_menu: "rules",
    };

    html.render().map(Html).map_err(|e| {
        eprintln!("Template render error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn activate_rule_handler(
    Path((account_id, rule_id)): Path<(i32, i32)>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    rules::activate_rule_for_account(&db, account_id, rule_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

pub async fn deactivate_rule_handler(
    Path((account_id, rule_id)): Path<(i32, i32)>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    rules::deactivate_rule_for_account(&db, account_id, rule_id)
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
        Some(s) if !s.is_empty() => {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?
                .to_string()
        }
        _ => String::new(),
    };

    let date_end = match &form.date_end {
        Some(s) if !s.is_empty() => {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?
                .to_string()
        }
        _ => String::new(),
    };

    let inserted_rule = rules::create_rule(
        &db,
        form.name,
        form.label,
        form.percentage,
        form.category_id,
        form.regexpr,
        date_start,
        date_end,
    )
    .await
    .map_err(|e| {
        eprintln!("Error creating rule: {:?}", e);
        axum::http::StatusCode::BAD_REQUEST
    })?;

    rules::activate_rule_for_account(&db, account_id, inserted_rule.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to(&format!("/accounts/{}/rules", account_id)))
}

fn get_applicable_rules(
    transaction: transaction::Model,
    rules: Vec<rule::Model>,
) -> Vec<rule::Model> {
    let mut appliers: Vec<rule::Model> = vec![];

    'rules: for rule in rules {
        if rule.regexpr.is_some() {
            let regexprs: Vec<&str> = rule.regexpr.as_deref().unwrap_or("").split(',').collect();

            for regexpr in regexprs {
                let re = match Regex::new(regexpr) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Skipping invalid regex {:?}: {}", regexpr, e);
                        continue;
                    }
                };
                if re.is_match(&transaction.description) {
                    appliers.push(rule.clone());
                    continue 'rules;
                }
            }
        }

        if rule.date_start.is_some() && rule.date_end.is_some() {
            let date_start: Option<NaiveDateTime> =
                rule.date_start.and_then(|d| d.and_hms_opt(0, 0, 0));

            let date_end: Option<NaiveDateTime> =
                rule.date_end.and_then(|d| d.and_hms_opt(23, 59, 59));

            if (date_start.map_or(true, |start| transaction.date >= start))
                && (date_end.map_or(true, |end| transaction.date <= end))
            {
                appliers.push(rule.clone());
            }
        }
    }

    appliers
}

pub async fn preview_apply_rules(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<Vec<PreviewTransaction>>, StatusCode> {
    let uncategorized_transactions =
        transactions::get_uncategorized_transactions_for_account(&db, account_id)
            .await
            .map_err(|e| {
                eprintln!("Error reading uncategorized transactions: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let active_rules = rules::get_active_rules_for_account(&db, account_id)
        .await
        .map_err(|e| {
            eprintln!("Error reading active rules: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // P3: Batch-load all categories once to avoid N+1 lookups inside the loop
    let all_categories = categories::get_categories(&db).await.map_err(|e| {
        eprintln!("Error loading categories: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let category_map: HashMap<i32, String> = all_categories
        .into_iter()
        .map(|c| (c.id, c.category))
        .collect();

    let mut previews: Vec<PreviewTransaction> = Vec::new();

    for transaction in uncategorized_transactions {
        let applicable_rules = get_applicable_rules(transaction.clone(), active_rules.clone());
        let mut category_new_value: String = String::new();
        let mut category_old_value: String = String::new();
        let mut new_percentage: f32 = transaction.perc_to_exclude;
        let mut new_label: String = String::new();

        if applicable_rules.is_empty() {
            continue;
        } else if applicable_rules.len() == 1 {
            let the_rule = &applicable_rules[0];

            category_old_value = transaction
                .category_id
                .and_then(|id| category_map.get(&id))
                .cloned()
                .unwrap_or_default();

            category_new_value = category_map
                .get(&the_rule.category_id)
                .cloned()
                .unwrap_or_default();

            new_percentage = the_rule.percentage;
            new_label = the_rule.label.clone();
        }

        previews.push(PreviewTransaction {
            id: transaction.id,
            description: transaction.description,
            value: transaction.value,
            date: transaction.date.to_string(),
            conflicts: applicable_rules,
            label_old_value: transaction.label,
            label_new_value: new_label,
            perc_to_exclude_old_value: transaction.perc_to_exclude,
            perc_to_exclude_new_value: new_percentage,
            category_old_value,
            category_new_value,
        });
    }

    Ok(Json(previews))
}

pub async fn apply_rules(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    let uncategorized_transactions =
        transactions::get_uncategorized_transactions_for_account(&db, account_id)
            .await
            .map_err(|e| {
                eprintln!("Error reading uncategorized transactions: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let active_rules = rules::get_active_rules_for_account(&db, account_id)
        .await
        .map_err(|e| {
            eprintln!("Error reading active rules: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    for tx in uncategorized_transactions {
        let applicable_rules = get_applicable_rules(tx.clone(), active_rules.clone());

        if applicable_rules.len() == 1 {
            let the_rule = &applicable_rules[0];
            let mut the_transaction: transaction::ActiveModel = tx.into();
            the_transaction.label = Set(the_rule.label.clone());
            the_transaction.perc_to_exclude = Set(the_rule.percentage);
            the_transaction.category_id = Set(Some(the_rule.category_id));

            the_transaction.update(&db).await.map_err(|err| {
                eprint!("Cannot update transaction: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    Ok(StatusCode::OK)
}

pub async fn resolve_conflicts_rules(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Json(payload): Json<Vec<ResolveConflictPayload>>,
) -> impl IntoResponse {
    let active_rules = match rules::get_active_rules_for_account(&db, account_id).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error reading active rules: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    for item in payload {
        let tx = match transaction::Entity::find_by_id(item.transaction_id)
            .one(&db)
            .await
        {
            Ok(Some(t)) => t,
            Ok(None) => return StatusCode::NOT_FOUND,
            Err(e) => {
                eprintln!("Error reading transaction {}: {:?}", item.transaction_id, e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };
        let applicable_rules = get_applicable_rules(tx.clone(), active_rules.clone());

        if applicable_rules.len() <= 1 || !applicable_rules.iter().any(|r| r.id == item.rule_id) {
            return StatusCode::NOT_FOUND;
        }

        let the_rule: rule::Model = match rule::Entity::find_by_id(item.rule_id).one(&db).await {
            Ok(Some(r)) => r,
            Ok(None) => return StatusCode::NOT_FOUND,
            Err(e) => {
                eprintln!("Error reading rule {}: {:?}", item.rule_id, e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };
        let mut the_transaction: transaction::ActiveModel = tx.into();

        the_transaction.label = Set(the_rule.label.clone());
        the_transaction.perc_to_exclude = Set(the_rule.percentage);
        the_transaction.category_id = Set(Some(the_rule.category_id));

        if let Err(err) = the_transaction.update(&db).await {
            eprintln!("Cannot update transaction: {}", err);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    StatusCode::OK
}
