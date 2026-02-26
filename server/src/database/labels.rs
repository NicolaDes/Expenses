use crate::database::entities::label;
use anyhow::Context;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};

pub async fn get_all_labels(db: &DatabaseConnection) -> anyhow::Result<Vec<label::Model>> {
    let models = label::Entity::find()
        .order_by_asc(label::Column::Name)
        .all(db)
        .await?;
    Ok(models)
}

pub async fn create_label(
    db: &DatabaseConnection,
    name: String,
) -> anyhow::Result<label::Model> {
    let active_model = label::ActiveModel {
        name: Set(name),
        ..Default::default()
    };
    let model = label::Entity::insert(active_model)
        .exec_with_returning(db)
        .await
        .context("Failed to insert new label into database")?;
    Ok(model)
}

pub async fn delete_label(db: &DatabaseConnection, id: i32) -> anyhow::Result<()> {
    // Check if any transaction references this label
    use crate::database::entities::transaction;
    let tx_count = transaction::Entity::find()
        .filter(transaction::Column::LabelId.eq(id))
        .count(db)
        .await
        .context("Failed to count transactions referencing label")?;

    if tx_count > 0 {
        anyhow::bail!("Cannot delete label: {} transaction(s) reference it", tx_count);
    }

    // Check if any rule references this label
    use crate::database::entities::rule;
    let rule_count = rule::Entity::find()
        .filter(rule::Column::LabelId.eq(id))
        .count(db)
        .await
        .context("Failed to count rules referencing label")?;

    if rule_count > 0 {
        anyhow::bail!("Cannot delete label: {} rule(s) reference it", rule_count);
    }

    let deleted = label::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context("Failed to delete label")?;

    if deleted.rows_affected == 0 {
        anyhow::bail!("Label not found");
    }

    Ok(())
}

pub async fn get_label(
    db: &DatabaseConnection,
    id: i32,
) -> anyhow::Result<Option<label::Model>> {
    let model = label::Entity::find_by_id(id).one(db).await?;
    Ok(model)
}

pub async fn get_label_by_name(
    db: &DatabaseConnection,
    name: &str,
) -> anyhow::Result<Option<label::Model>> {
    let model = label::Entity::find()
        .filter(label::Column::Name.eq(name))
        .one(db)
        .await?;
    Ok(model)
}
