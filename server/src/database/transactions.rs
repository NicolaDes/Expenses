use crate::database::entities::transaction;
use anyhow::Context;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

// pub async fn get_transactions(db: &DatabaseConnection) -> anyhow::Result<Vec<transaction::Model>> {
//     let transactions = transaction::Entity::find().all(db).await?;
//     Ok(transactions)
// }

// pub async fn create_transaction(
//     db: &DatabaseConnection,

//     account_id: i32,
//     category_id: Option<i32>,
//     value: f64,
//     description: String,
//     date: chrono::NaiveDateTime,
//     perc_to_exclude: f32,
//     label: String,
// ) -> anyhow::Result<transaction::Model> {
//     let active_model = transaction::ActiveModel {
//         account_id: Set(account_id),
//         category_id: Set(category_id),
//         value: Set(value),
//         description: Set(description),
//         date: Set(date),
//         perc_to_exclude: Set(perc_to_exclude),
//         label: Set(label),
//         ..Default::default()
//     };

//     let model = transaction::Entity::insert(active_model)
//         .exec_with_returning(db)
//         .await
//         .context("Failed to insert new transaction into database")?;

//     Ok(model)
// }

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

pub async fn edit_transaction(
    db: &DatabaseConnection,
    id: i32,
    account_id: i32,
    category_id: Option<i32>,
    value: f64,
    description: String,
    date: String,
    perc_to_exclude: f32,
    label: String,
) -> anyhow::Result<transaction::Model> {
    let mut active_model: transaction::ActiveModel = transaction::Entity::find_by_id(id)
        .one(db)
        .await
        .expect("Error reading the transaction!")
        .unwrap()
        .into();

    active_model.account_id = Set(account_id);
    active_model.category_id = Set(category_id);
    active_model.value = Set(value);
    active_model.description = Set(description);
    match chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%dT%H:%M") {
        Ok(date) => active_model.date = Set(date.into()),
        Err(err) => {
            anyhow::bail!("Error parsing date '{}': {}", date, err);
        }
    }

    active_model.perc_to_exclude = Set(perc_to_exclude);
    active_model.label = Set(label);

    let model = transaction::Entity::update(active_model)
        .exec(db)
        .await
        .context("Error updating the transaction!")?;

    Ok(model)
}
