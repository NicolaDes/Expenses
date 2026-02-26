use crate::database::{account, entities::budget};
use anyhow::Context;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, ModelTrait};

pub async fn get_budgets_for_account(
    db: &DatabaseConnection,
    account_id: i32,
) -> anyhow::Result<Vec<budget::Model>> {
    let account = account::Entity::find_by_id(account_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Account not found"))?;

    let budgets: Vec<budget::Model> = account.find_related(budget::Entity).all(db).await?;

    Ok(budgets)
}

pub async fn create_budget(
    db: &DatabaseConnection,
    account_id: i32,
    name: String,
    value: f64,
) -> anyhow::Result<budget::Model> {
    let active_model = budget::ActiveModel {
        account_id: Set(account_id),
        name: Set(name),
        value: Set(value),
        ..Default::default()
    };

    let model = budget::Entity::insert(active_model)
        .exec_with_returning(db)
        .await
        .context("Failed to insert new budget into database")?;

    Ok(model)
}

pub async fn delete_budget(db: &DatabaseConnection, id: i32) -> anyhow::Result<()> {
    let deleted = budget::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context("Failed to delete budget!")?;

    if deleted.rows_affected == 0 {
        anyhow::bail!("budget not found");
    }

    Ok(())
}

pub async fn edit_budget(
    db: &DatabaseConnection,
    id: i32,
    account_id: i32,
    name: String,
    value: f64,
) -> anyhow::Result<budget::Model> {
    let mut active_model: budget::ActiveModel = budget::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Budget {} not found", id))?
        .into();

    active_model.account_id = Set(account_id);
    active_model.name = Set(name);
    active_model.value = Set(value);

    let model = budget::Entity::update(active_model)
        .exec(db)
        .await
        .context("Error updating the budget!")?;

    Ok(model)
}
