use crate::database::{category, entities::settings};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};

// pub async fn get_settingss(db: &DatabaseConnection) -> anyhow::Result<Vec<settings::Model>> {
//     let settingss = settings::Entity::find().all(db).await?;
//     Ok(settingss)
// }

pub async fn get_settings_for_account(
    db: &DatabaseConnection,
    account_id: i32,
) -> anyhow::Result<settings::Model> {
    let settings = settings::Entity::find()
        .filter(settings::Column::AccountId.eq(account_id))
        .one(db)
        .await?
        .unwrap();

    Ok(settings)
}

pub async fn get_excluded_categories_for_account(
    db: &DatabaseConnection,
    account_id: i32,
) -> anyhow::Result<Vec<category::Model>> {
    let categories = get_settings_for_account(&db, account_id)
        .await
        .unwrap()
        .find_related(category::Entity)
        .all(db)
        .await?;

    Ok(categories)
}

// pub async fn create_settings(
//     db: &DatabaseConnection,
//     account_id: i32,
//     date_index: i32,
//     description_index: i32,
//     value_index: i32,
//     starter_string: String,
//     report_delimiter: String,
//     report_decimal_separator: String,
// ) -> anyhow::Result<settings::Model> {
//     let active_model = settings::ActiveModel {
//         account_id: Set(account_id),
//         date_index: Set(date_index),
//         description_index: Set(description_index),
//         value_index: Set(value_index),
//         starter_string: Set(starter_string),
//         report_delimiter: Set(report_delimiter),
//         report_decimal_separator: Set(report_decimal_separator),
//         ..Default::default()
//     };

//     let model = settings::Entity::insert(active_model)
//         .exec_with_returning(db)
//         .await
//         .context("Failed to insert new settings into database")?;

//     Ok(model)
// }

// pub async fn delete_settings(db: &DatabaseConnection, id: i32) -> anyhow::Result<()> {
//     let deleted = settings::Entity::delete_by_id(id)
//         .exec(db)
//         .await
//         .context("Failed to delete settings!")?;

//     if deleted.rows_affected == 0 {
//         anyhow::bail!("settings not found");
//     }

//     Ok(())
// }

// pub async fn edit_settings(
//     db: &DatabaseConnection,
//     id: i32,
//     account_id: i32,
//     date_index: i32,
//     description_index: i32,
//     value_index: i32,
//     starter_string: String,
//     report_delimiter: String,
//     report_decimal_separator: String,
// ) -> anyhow::Result<settings::Model> {
//     let mut active_model: settings::ActiveModel = settings::Entity::find_by_id(id)
//         .one(db)
//         .await
//         .expect("Error reading the settings!")
//         .unwrap()
//         .into();

//     active_model.account_id = Set(account_id);
//     active_model.date_index = Set(date_index);
//     active_model.description_index = Set(description_index);
//     active_model.value_index = Set(value_index);
//     active_model.starter_string = Set(starter_string);
//     active_model.report_delimiter = Set(report_delimiter);
//     active_model.report_decimal_separator = Set(report_decimal_separator);

//     let model = settings::Entity::update(active_model)
//         .exec(db)
//         .await
//         .context("Error updating the settings!")?;

//     Ok(model)
// }
