use crate::database::{
    category,
    entities::{settings, settings_excluded_category},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};

pub async fn get_settings_for_account(
    db: &DatabaseConnection,
    account_id: i32,
) -> anyhow::Result<settings::Model> {
    let settings = match settings::Entity::find()
        .filter(settings::Column::AccountId.eq(account_id))
        .one(db)
        .await?
    {
        Some(s) => s,
        None => {
            settings::ActiveModel {
                account_id: Set(account_id),
                date_index: Set(0),
                description_index: Set(0),
                value_index: Set(0),
                starter_string: Set("".to_string()),
                report_delimiter: Set(";".to_string()),
                report_decimal_separator: Set(",".to_string()),
                ..Default::default()
            }
            .insert(db)
            .await?
        }
    };

    Ok(settings)
}

pub async fn get_excluded_categories_for_account(
    db: &DatabaseConnection,
    account_id: i32,
) -> anyhow::Result<Vec<category::Model>> {
    let categories = get_settings_for_account(db, account_id)
        .await?
        .find_related(category::Entity)
        .all(db)
        .await?;

    Ok(categories)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_settings(
    db: &DatabaseConnection,
    account_id: i32,
    date_index: i32,
    description_index: i32,
    value_index: i32,
    starter_string: String,
    report_delimiter: String,
    report_decimal_separator: String,
) -> anyhow::Result<settings::Model> {
    let mut active_model: settings::ActiveModel = settings::Entity::find()
        .filter(settings::Column::AccountId.eq(account_id))
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Settings not found for account {}", account_id))?
        .into();

    active_model.date_index = Set(date_index);
    active_model.description_index = Set(description_index);
    active_model.value_index = Set(value_index);
    active_model.starter_string = Set(starter_string);
    active_model.report_delimiter = Set(report_delimiter);
    active_model.report_decimal_separator = Set(report_decimal_separator);

    let model = active_model.update(db).await?;
    Ok(model)
}

pub async fn add_excluded_category(
    db: &DatabaseConnection,
    account_id: i32,
    category_id: i32,
) -> anyhow::Result<()> {
    let settings = get_settings_for_account(db, account_id).await?;
    settings_excluded_category::ActiveModel {
        settings_id: Set(settings.id),
        category_id: Set(category_id),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(())
}

pub async fn delete_excluded_category(
    db: &DatabaseConnection,
    account_id: i32,
    category_id: i32,
) -> anyhow::Result<()> {
    let settings = get_settings_for_account(db, account_id).await?;
    settings_excluded_category::Entity::delete_many()
        .filter(settings_excluded_category::Column::SettingsId.eq(settings.id))
        .filter(settings_excluded_category::Column::CategoryId.eq(category_id))
        .exec(db)
        .await?;
    Ok(())
}
