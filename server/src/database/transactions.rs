use crate::database::{category, entities::transaction};
use anyhow::Context;
use chrono::NaiveDate;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};

#[allow(clippy::too_many_arguments)]
pub async fn create_transaction(
    db: &DatabaseConnection,
    account_id: i32,
    category_id: Option<i32>,
    value: f64,
    description: String,
    date: chrono::NaiveDateTime,
    perc_to_exclude: f32,
    label_id: Option<i32>,
) -> anyhow::Result<transaction::Model> {
    let active_model = transaction::ActiveModel {
        account_id: Set(account_id),
        category_id: Set(category_id),
        value: Set(value),
        description: Set(description),
        date: Set(date),
        perc_to_exclude: Set(perc_to_exclude),
        label_id: Set(label_id),
        ..Default::default()
    };

    let model = transaction::Entity::insert(active_model)
        .exec_with_returning(db)
        .await
        .context("Failed to insert new transaction into database")?;

    Ok(model)
}

pub async fn delete_transaction(db: &DatabaseConnection, id: i32) -> anyhow::Result<()> {
    let deleted = transaction::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context("Failed to delete transaction!")?;

    if deleted.rows_affected == 0 {
        anyhow::bail!("Transaction not found");
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn edit_transaction(
    db: &DatabaseConnection,
    id: i32,
    account_id: i32,
    category_id: Option<i32>,
    value: f64,
    description: String,
    date: String,
    perc_to_exclude: f32,
    label_id: Option<i32>,
) -> anyhow::Result<transaction::Model> {
    let mut active_model: transaction::ActiveModel = transaction::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Transaction {} not found", id))?
        .into();

    active_model.account_id = Set(account_id);
    active_model.category_id = Set(category_id);
    active_model.value = Set(value);
    active_model.description = Set(description);
    match chrono::NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M") {
        Ok(dt) => active_model.date = Set(dt),
        Err(err) => {
            anyhow::bail!("Error parsing date '{}': {}", date, err);
        }
    }

    active_model.perc_to_exclude = Set(perc_to_exclude);
    active_model.label_id = Set(label_id);

    let model = transaction::Entity::update(active_model)
        .exec(db)
        .await
        .context("Error updating the transaction!")?;

    Ok(model)
}

pub async fn get_transactions_with_categories_for_account(
    db: &DatabaseConnection,
    account_id: i32,
) -> anyhow::Result<Vec<(transaction::Model, Vec<category::Model>)>> {
    let result = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .find_with_related(category::Entity)
        .all(db)
        .await?;
    Ok(result)
}

pub async fn get_uncategorized_transactions_for_account(
    db: &DatabaseConnection,
    account_id: i32,
) -> anyhow::Result<Vec<transaction::Model>> {
    let result = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .filter(transaction::Column::CategoryId.is_null())
        .all(db)
        .await?;
    Ok(result)
}

#[allow(dead_code)]
pub async fn get_transactions_from_year_start(
    db: &DatabaseConnection,
    account_id: i32,
    start_of_year: NaiveDate,
) -> anyhow::Result<Vec<transaction::Model>> {
    let result = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .filter(transaction::Column::Date.gt(start_of_year))
        .order_by_asc(transaction::Column::Date)
        .all(db)
        .await?;
    Ok(result)
}
