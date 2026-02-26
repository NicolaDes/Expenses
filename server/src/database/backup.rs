use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::database::{
    account, account_rule, budget, category, rule, settings, settings_excluded_category,
    transaction,
};

#[derive(Serialize, Deserialize)]
pub struct FullBackupDTO {
    pub accounts: Vec<AccountDTO>,
    pub budgets: Vec<BudgetDTO>,
    pub transactions: Vec<TransactionDTO>,
    pub rules: Vec<RuleDTO>,
    pub categories: Vec<CategoryDTO>,
    pub account_rules: Vec<AccountRuleDTO>,
    pub settings: Vec<AccountSettingsDTO>,
    #[serde(rename = "exlcuded_categories")]
    pub excluded_categories: Vec<SettingsExcludedCategoriesDTO>,
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
    pub report_delimiter: String,
    pub report_decimal_separator: String,
}

#[derive(Serialize, Deserialize)]
pub struct SettingsExcludedCategoriesDTO {
    pub id: i32,
    pub settings_id: i32,
    pub category_id: i32,
}

pub async fn get_full_backup(db: &DatabaseConnection) -> anyhow::Result<String> {
    let accounts_model = account::Entity::find().all(db).await?;
    let accounts_dto: Vec<AccountDTO> = accounts_model
        .into_iter()
        .map(|a| AccountDTO {
            id: a.id,
            name: a.name,
        })
        .collect();

    let budgets_model = budget::Entity::find().all(db).await?;
    let budgets_dto: Vec<BudgetDTO> = budgets_model
        .into_iter()
        .map(|b| BudgetDTO {
            id: b.id,
            account_id: b.account_id,
            name: b.name,
            value: b.value,
        })
        .collect();

    let transactions_model = transaction::Entity::find().all(db).await?;
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

    let rules_model = rule::Entity::find().all(db).await?;
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

    let categories_model = category::Entity::find().all(db).await?;
    let categories_dto: Vec<CategoryDTO> = categories_model
        .into_iter()
        .map(|c| CategoryDTO {
            id: c.id,
            transaction_type: c.transaction_type,
            macro_category: c.macro_category,
            category: c.category,
        })
        .collect();

    let account_rules_model = account_rule::Entity::find().all(db).await?;
    let account_rules_dto: Vec<AccountRuleDTO> = account_rules_model
        .into_iter()
        .map(|ar| AccountRuleDTO {
            id: ar.id,
            account_id: ar.account_id,
            rule_id: ar.rule_id,
        })
        .collect();

    let settings_model = settings::Entity::find().all(db).await?;
    let settings_dto: Vec<AccountSettingsDTO> = settings_model
        .into_iter()
        .map(|s| AccountSettingsDTO {
            id: s.id,
            account_id: s.account_id,
            date_index: s.date_index,
            description_index: s.description_index,
            value_index: s.value_index,
            starter_string: s.starter_string,
            report_delimiter: s.report_delimiter,
            report_decimal_separator: s.report_decimal_separator,
        })
        .collect();

    let excluded_categories_model = settings_excluded_category::Entity::find().all(db).await?;
    let excluded_categories_dto: Vec<SettingsExcludedCategoriesDTO> = excluded_categories_model
        .into_iter()
        .map(|ec| SettingsExcludedCategoriesDTO {
            id: ec.id,
            settings_id: ec.settings_id,
            category_id: ec.category_id,
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
        excluded_categories: excluded_categories_dto,
    };

    let json_backup = serde_json::to_string_pretty(&backup)?;
    Ok(json_backup)
}
