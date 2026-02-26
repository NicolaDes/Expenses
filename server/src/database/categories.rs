use crate::database::entities::category;
use anyhow::Context;
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn get_categories(db: &DatabaseConnection) -> anyhow::Result<Vec<category::Model>> {
    let models = category::Entity::find().all(db).await?;
    Ok(models)
}

#[allow(dead_code)]
pub async fn get_categories_excluding(
    db: &DatabaseConnection,
    excluded_ids: Vec<i32>,
) -> anyhow::Result<Vec<category::Model>> {
    let models = category::Entity::find()
        .filter(category::Column::Id.is_not_in(excluded_ids))
        .all(db)
        .await?;
    Ok(models)
}

#[allow(dead_code)]
pub async fn get_categories_by_names(
    db: &DatabaseConnection,
    names: Vec<String>,
) -> anyhow::Result<Vec<category::Model>> {
    let models = category::Entity::find()
        .filter(category::Column::Category.is_in(names))
        .all(db)
        .await?;
    Ok(models)
}

#[allow(dead_code)]
pub async fn get_category(
    db: &DatabaseConnection,
    id: i32,
) -> anyhow::Result<category::Model> {
    let model = category::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Category {} not found", id))?;
    Ok(model)
}

pub async fn create_category(
    db: &DatabaseConnection,
    category: String,
    macro_category: String,
    transaction_type: String,
) -> anyhow::Result<category::Model> {
    let active_model: category::ActiveModel = category::ActiveModel {
        category: Set(category),
        macro_category: Set(macro_category),
        transaction_type: Set(transaction_type),
        ..Default::default()
    };

    let model: category::Model = category::Entity::insert(active_model)
        .exec_with_returning(db)
        .await
        .context("Failed to insert new category into database")?;

    Ok(model)
}

pub async fn delete_category(db: &DatabaseConnection, id: i32) -> anyhow::Result<()> {
    let deleted = category::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context("Failed to delete category!")?;

    if deleted.rows_affected == 0 {
        anyhow::bail!("category not found");
    }

    Ok(())
}

pub async fn edit_category(
    db: &DatabaseConnection,
    id: i32,
    category: String,
    macro_category: String,
    transaction_type: String,
) -> anyhow::Result<category::Model> {
    let mut active_model: category::ActiveModel = category::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Category {} not found", id))?
        .into();

    active_model.transaction_type = Set(transaction_type);
    active_model.macro_category = Set(macro_category);
    active_model.category = Set(category);

    let model = category::Entity::update(active_model)
        .exec(db)
        .await
        .context("Error updating the category!")?;

    Ok(model)
}
