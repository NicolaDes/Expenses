use crate::database::entities::account;
use sea_orm::{DatabaseConnection, EntityTrait};

pub async fn get_all_accounts(db: &DatabaseConnection) -> anyhow::Result<Vec<account::Model>> {
    let accounts = account::Entity::find().all(db).await?;
    Ok(accounts)
}

// TODO: move here the creation of a new account
