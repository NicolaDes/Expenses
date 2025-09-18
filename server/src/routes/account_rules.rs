use crate::database::{
    category,
    entities::{account, account_rule, rule},
    transaction,
};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Form, Json,
};
use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "account_rules.html")]
struct AccountRulesTemplate<'a> {
    account: account::Model,
    active_rules: Vec<rule::Model>,
    inactive_rules: Vec<rule::Model>,
    categories: Vec<category::Model>,
    uncategorized_transactions: Vec<transaction::Model>,
    menu: &'a str,
    sub_menu: &'a str,
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

    let uncategorized_transactions = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .filter(transaction::Column::CategoryId.is_null())
        .all(&db)
        .await
        .map_err(|err| {
            eprintln!("Errore filter by null: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let html = AccountRulesTemplate {
        account: account_data,
        active_rules,
        inactive_rules,
        categories,
        uncategorized_transactions,
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

fn get_applayable_rules(
    transaction: transaction::Model,
    rules: Vec<rule::Model>,
) -> Vec<rule::Model> {
    let mut appliers: Vec<rule::Model> = vec![];

    'rules: for rule in rules {
        if !rule.regexpr.is_none() {
            let regexprs: Vec<&str> = rule.regexpr.as_deref().unwrap_or("").split(',').collect();

            for regexpr in regexprs {
                let re = Regex::new(regexpr).unwrap();
                if re.is_match(&transaction.description) {
                    appliers.push(rule.clone());
                    continue 'rules;
                }
            }
        }

        if !rule.date_start.is_none() && !rule.date_end.is_none() {
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

    return appliers;
}

pub async fn preview_apply_rules(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Json<Vec<PreviewTransaction>> {
    let mut previews: Vec<PreviewTransaction> = Vec::new();

    let uncategorized_transactions = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .filter(transaction::Column::CategoryId.is_null())
        .all(&db)
        .await
        .expect("Errore nella lettura delle transazioni non categorizzate!");

    let active_rules_raw = account::Entity::find_by_id(account_id)
        .find_with_related(rule::Entity)
        .all(&db)
        .await
        .expect("Errore nel leggere le regole attive");

    let active_rules: Vec<rule::Model> = active_rules_raw
        .into_iter()
        .flat_map(|(_acc, rules)| rules)
        .collect();

    for transaction in uncategorized_transactions {
        let applicable_rules = get_applayable_rules(transaction.clone(), active_rules.clone());
        let mut category_new_value: String = String::new();
        let mut category_old_value: String = String::new();
        let mut new_percentage: f32 = transaction.perc_to_exclude;
        let mut new_label: String = String::new();

        if applicable_rules.len() == 0 {
            continue;
        } else if applicable_rules.len() == 1 {
            let the_rule = &applicable_rules[0];

            category_old_value = match transaction.category_id {
                Some(cat_id) => category::Entity::find_by_id(cat_id)
                    .one(&db)
                    .await
                    .expect(&format!("Cannot find category with id {}", cat_id))
                    .map(|c| c.category)
                    .unwrap_or_default(),
                None => String::new(),
            };

            category_new_value = category::Entity::find_by_id(the_rule.category_id)
                .one(&db)
                .await
                .expect(&format!(
                    "Cannot find category with id {}",
                    the_rule.category_id
                ))
                .map(|c| c.category)
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

    Json(previews)
}

pub async fn apply_rules(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    let uncategorized_transactions = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .filter(transaction::Column::CategoryId.is_null())
        .all(&db)
        .await
        .expect("Errore nella lettura delle transazioni non categorizzate!");

    let active_rules_raw = account::Entity::find_by_id(account_id)
        .find_with_related(rule::Entity)
        .all(&db)
        .await
        .expect("Errore nel leggere le regole attive");

    let active_rules: Vec<rule::Model> = active_rules_raw
        .into_iter()
        .flat_map(|(_acc, rules)| rules)
        .collect();

    for transaction in uncategorized_transactions {
        let applicable_rules = get_applayable_rules(transaction.clone(), active_rules.clone());

        if applicable_rules.len() == 1 {
            let the_rule = &applicable_rules[0];
            let mut the_transaction: transaction::ActiveModel = transaction.into();
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
    let active_rules_raw = account::Entity::find_by_id(account_id)
        .find_with_related(rule::Entity)
        .all(&db)
        .await
        .expect("Errore nel leggere le regole attive");

    let active_rules: Vec<rule::Model> = active_rules_raw
        .into_iter()
        .flat_map(|(_acc, rules)| rules)
        .collect();

    for item in payload {
        let transaction = transaction::Entity::find_by_id(item.transaction_id)
            .all(&db)
            .await
            .expect("Errore nella lettura della transazione")[0]
            .clone();
        let applicable_rules = get_applayable_rules(transaction.clone(), active_rules.clone());

        if applicable_rules.len() <= 1 || !applicable_rules.iter().any(|r| r.id == item.rule_id) {
            return StatusCode::NOT_FOUND;
        }

        let the_rule: rule::Model = rule::Entity::find_by_id(item.rule_id)
            .all(&db)
            .await
            .expect("Errore nella lettura della regola")[0]
            .clone();
        let mut the_transaction: transaction::ActiveModel = transaction.into();

        the_transaction.label = Set(the_rule.label.clone());
        the_transaction.perc_to_exclude = Set(the_rule.percentage);
        the_transaction.category_id = Set(Some(the_rule.category_id));

        let _ =the_transaction.update(&db).await.map_err(|err| {
            eprint!("Cannot update transaction: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        });
    }

    return StatusCode::OK;
}
