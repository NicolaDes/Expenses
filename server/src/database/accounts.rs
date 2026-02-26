use crate::database::entities::account;
use anyhow::Context;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

pub async fn get_accounts(db: &DatabaseConnection) -> anyhow::Result<Vec<account::Model>> {
    let models = account::Entity::find().all(db).await?;
    Ok(models)
}

pub async fn get_account(db: &DatabaseConnection, id: i32) -> anyhow::Result<account::Model> {
    let model = account::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Account {} not found", id))?;
    Ok(model)
}

pub async fn create_account(
    db: &DatabaseConnection,
    name: String,
) -> anyhow::Result<account::Model> {
    let active_model: account::ActiveModel = account::ActiveModel {
        name: Set(name),
        ..Default::default()
    };

    let model: account::Model = account::Entity::insert(active_model)
        .exec_with_returning(db)
        .await
        .context("Failed to insert new account into database")?;

    Ok(model)
}

pub async fn delete_account(db: &DatabaseConnection, id: i32) -> anyhow::Result<()> {
    let deleted = account::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context("Failed to delete account!")?;

    if deleted.rows_affected == 0 {
        anyhow::bail!("Account not found");
    }

    Ok(())
}
