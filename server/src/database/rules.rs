use crate::database::{
    category,
    entities::{account, account_rule, rule},
};
use anyhow::Context;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter,
};
use std::collections::HashSet;

pub async fn get_rules(db: &DatabaseConnection) -> anyhow::Result<Vec<rule::Model>> {
    let rules = rule::Entity::find().all(db).await?;
    Ok(rules)
}

pub async fn get_rules_with_categories(
    db: &DatabaseConnection,
) -> anyhow::Result<Vec<(rule::Model, Vec<category::Model>)>> {
    let rules_with_cats = rule::Entity::find()
        .find_with_related(category::Entity)
        .all(db)
        .await?;

    Ok(rules_with_cats)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_rule(
    db: &DatabaseConnection,
    name: String,
    label_id: i32,
    percentage: f32,
    category_id: i32,
    regexpr: Option<String>,
    date_start: String,
    date_end: String,
) -> anyhow::Result<rule::Model> {
    let active_model = rule::ActiveModel {
        name: Set(name),
        label_id: Set(label_id),
        percentage: Set(percentage),
        category_id: Set(category_id),
        regexpr: Set(regexpr),
        date_start: Set(if date_start.trim().is_empty() {
            None
        } else {
            match chrono::NaiveDate::parse_from_str(&date_start, "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(err) => anyhow::bail!("Error parsing date '{}': {}", date_start, err),
            }
        }),
        date_end: Set(if date_end.trim().is_empty() {
            None
        } else {
            match chrono::NaiveDate::parse_from_str(&date_end, "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(err) => anyhow::bail!("Error parsing date '{}': {}", date_end, err),
            }
        }),
        ..Default::default()
    };

    let model = rule::Entity::insert(active_model)
        .exec_with_returning(db)
        .await
        .context("Failed to insert new rule into database")?;

    Ok(model)
}

pub async fn delete_rule(db: &DatabaseConnection, id: i32) -> anyhow::Result<()> {
    let deleted = rule::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context("Failed to delete rule!")?;

    if deleted.rows_affected == 0 {
        anyhow::bail!("rule not found");
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn edit_rule(
    db: &DatabaseConnection,
    id: i32,
    name: String,
    label_id: i32,
    percentage: f32,
    category_id: i32,
    regexpr: Option<String>,
    date_start: String,
    date_end: String,
) -> anyhow::Result<rule::Model> {
    let mut active_model: rule::ActiveModel = rule::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Rule {} not found", id))?
        .into();

    active_model.name = Set(name);
    active_model.label_id = Set(label_id);
    active_model.percentage = Set(percentage);
    active_model.category_id = Set(category_id);
    active_model.regexpr = Set(regexpr);
    active_model.date_start = Set(if date_start.trim().is_empty() {
        None
    } else {
        match chrono::NaiveDate::parse_from_str(&date_start, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(err) => anyhow::bail!("Error parsing date '{}': {}", date_start, err),
        }
    });

    active_model.date_end = Set(if date_end.trim().is_empty() {
        None
    } else {
        match chrono::NaiveDate::parse_from_str(&date_end, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(err) => anyhow::bail!("Error parsing date '{}': {}", date_end, err),
        }
    });

    let model = rule::Entity::update(active_model)
        .exec(db)
        .await
        .context("Error updating the rule!")?;

    Ok(model)
}

pub async fn get_active_rule_ids_for_account(
    db: &DatabaseConnection,
    account_id: i32,
) -> anyhow::Result<HashSet<i32>> {
    let ids = account_rule::Entity::find()
        .filter(account_rule::Column::AccountId.eq(account_id))
        .all(db)
        .await?
        .into_iter()
        .map(|ar| ar.rule_id)
        .collect();
    Ok(ids)
}

pub async fn activate_rule_for_account(
    db: &DatabaseConnection,
    account_id: i32,
    rule_id: i32,
) -> anyhow::Result<()> {
    account_rule::ActiveModel {
        account_id: Set(account_id),
        rule_id: Set(rule_id),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(())
}

pub async fn deactivate_rule_for_account(
    db: &DatabaseConnection,
    account_id: i32,
    rule_id: i32,
) -> anyhow::Result<()> {
    account_rule::Entity::delete_many()
        .filter(account_rule::Column::AccountId.eq(account_id))
        .filter(account_rule::Column::RuleId.eq(rule_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn get_active_rules_for_account(
    db: &DatabaseConnection,
    account_id: i32,
) -> anyhow::Result<Vec<rule::Model>> {
    let active_rules_raw = account::Entity::find_by_id(account_id)
        .find_with_related(rule::Entity)
        .all(db)
        .await?;
    let active_rules: Vec<rule::Model> = active_rules_raw
        .into_iter()
        .flat_map(|(_acc, rules)| rules)
        .collect();
    Ok(active_rules)
}
