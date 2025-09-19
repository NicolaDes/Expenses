use axum::http::StatusCode;
use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::database::{account, account_rule, budget, category, rule, settings, transaction};

#[derive(Serialize, Deserialize)]
pub struct FullBackupDTO {
    pub accounts: Vec<AccountDTO>,
    pub budgets: Vec<BudgetDTO>,
    pub transactions: Vec<TransactionDTO>,
    pub rules: Vec<RuleDTO>,
    pub categories: Vec<CategoryDTO>,
    pub account_rules: Vec<AccountRuleDTO>,
    pub settings: Vec<AccountSettingsDTO>,
}

#[derive(Serialize, Deserialize)]
pub struct AccountDTO {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct BudgetDTO {
    pub id: i32,
    pub account_id: i32,
    pub name: String,
    pub value: f64,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionDTO {
    pub id: i32,
    pub account_id: i32,
    pub category_id: Option<i32>,
    pub value: f64,
    pub description: String,
    pub date: DateTime<chrono::Utc>,
    pub perc_to_exclude: f32,
    pub label: String,
}

#[derive(Serialize, Deserialize)]
pub struct RuleDTO {
    pub id: i32,
    pub name: String,
    pub label: String,
    pub percentage: f32,
    pub category_id: i32,
    pub regexpr: Option<String>,
    pub date_start: Option<NaiveDate>,
    pub date_end: Option<NaiveDate>,
}

#[derive(Serialize, Deserialize)]
pub struct CategoryDTO {
    pub id: i32,
    pub transaction_type: String,
    pub macro_category: String,
    pub category: String,
}

#[derive(Serialize, Deserialize)]
pub struct AccountRuleDTO {
    pub id: i32,
    pub account_id: i32,
    pub rule_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct AccountSettingsDTO {
    pub id: i32,
    pub account_id: i32,
    pub date_index: i32,
    pub description_index: i32,
    pub value_index: i32,
    pub starter_string: String,
}

pub async fn get_full_backup(db: &DatabaseConnection) -> Result<String, StatusCode> {
    let accounts_model = account::Entity::find().all(db).await.map_err(|e| {
        eprintln!("Errore recuperando accounts: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let accounts_dto: Vec<AccountDTO> = accounts_model
        .into_iter()
        .map(|a| AccountDTO {
            id: a.id,
            name: a.name,
        })
        .collect();

    let budgets_model = budget::Entity::find().all(db).await.map_err(|e| {
        eprintln!("Errore recuperando budgets: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let budgets_dto: Vec<BudgetDTO> = budgets_model
        .into_iter()
        .map(|b| BudgetDTO {
            id: b.id,
            account_id: b.account_id,
            name: b.name,
            value: b.value,
        })
        .collect();

    let transactions_model = transaction::Entity::find().all(db).await.map_err(|e| {
        eprintln!("Errore recuperando transazioni: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let transactions_dto: Vec<TransactionDTO> = transactions_model
        .into_iter()
        .map(|t| TransactionDTO {
            id: t.id,
            account_id: t.account_id,
            category_id: t.category_id,
            value: t.value,
            description: t.description,
            date: DateTime::from_naive_utc_and_offset(t.date, Utc),
            perc_to_exclude: t.perc_to_exclude,
            label: t.label,
        })
        .collect();

    let rules_model = rule::Entity::find().all(db).await.map_err(|e| {
        eprintln!("Errore recuperando rules: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let rules_dto: Vec<RuleDTO> = rules_model
        .into_iter()
        .map(|r| RuleDTO {
            id: r.id,
            name: r.name,
            label: r.label,
            percentage: r.percentage,
            category_id: r.category_id,
            regexpr: r.regexpr,
            date_start: r.date_start,
            date_end: r.date_end,
        })
        .collect();

    let categories_model = category::Entity::find().all(db).await.map_err(|e| {
        eprintln!("Errore recuperando categories: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let categories_dto: Vec<CategoryDTO> = categories_model
        .into_iter()
        .map(|c| CategoryDTO {
            id: c.id,
            transaction_type: c.transaction_type,
            macro_category: c.macro_category,
            category: c.category,
        })
        .collect();

    let account_rules_model = account_rule::Entity::find().all(db).await.map_err(|e| {
        eprintln!("Errore recuperando account_rules: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let account_rules_dto: Vec<AccountRuleDTO> = account_rules_model
        .into_iter()
        .map(|ar| AccountRuleDTO {
            id: ar.id,
            account_id: ar.account_id,
            rule_id: ar.rule_id,
        })
        .collect();

    let settings_model = settings::Entity::find().all(db).await.map_err(|e| {
        eprintln!("Errore recuperando transazioni: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let settings_dto: Vec<AccountSettingsDTO> = settings_model
        .into_iter()
        .map(|account_setting| AccountSettingsDTO {
            id: account_setting.id,
            account_id: account_setting.account_id,
            date_index: account_setting.date_index,
            description_index: account_setting.description_index,
            value_index: account_setting.value_index,
            starter_string: account_setting.starter_string,
        })
        .collect();

    let backup = FullBackupDTO {
        accounts: accounts_dto,
        budgets: budgets_dto,
        transactions: transactions_dto,
        rules: rules_dto,
        categories: categories_dto,
        account_rules: account_rules_dto,
        settings: settings_dto,
    };

    let json_backup = serde_json::to_string_pretty(&backup).map_err(|e| {
        eprintln!("Errore serializzando JSON: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(json_backup)
}
