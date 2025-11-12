use crate::database::{category, entities::rule};
use anyhow::Context;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};

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

// pub async fn create_rule(
//     db: &DatabaseConnection,
//     name: String,
//     label: String,
//     percentage: f32,
//     category_id: i32,
//     regexpr: Option<String>,
//     date_start: String,
//     date_end: String,
// ) -> anyhow::Result<rule::Model> {
//     let active_model = rule::ActiveModel {
//         name: Set(name),
//         label: Set(label),
//         percentage: Set(percentage),
//         category_id: Set(category_id),
//         regexpr: Set(regexpr),
//         date_start: Set(if date_start.trim().is_empty() {
//             None
//         } else {
//             match chrono::NaiveDate::parse_from_str(&date_start, "%Y-%m-%d") {
//                 Ok(d) => Some(d),
//                 Err(err) => anyhow::bail!("Error parsing date '{}': {}", date_start, err),
//             }
//         }),
//         date_end: Set(if date_end.trim().is_empty() {
//             None
//         } else {
//             match chrono::NaiveDate::parse_from_str(&date_end, "%Y-%m-%d") {
//                 Ok(d) => Some(d),
//                 Err(err) => anyhow::bail!("Error parsing date '{}': {}", date_end, err),
//             }
//         }),
//         ..Default::default()
//     };

//     let model = rule::Entity::insert(active_model)
//         .exec_with_returning(db)
//         .await
//         .context("Failed to insert new rule into database")?;

//     Ok(model)
// }

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

pub async fn edit_rule(
    db: &DatabaseConnection,
    id: i32,
    name: String,
    label: String,
    percentage: f32,
    category_id: i32,
    regexpr: Option<String>,
    date_start: String,
    date_end: String,
) -> anyhow::Result<rule::Model> {
    let mut active_model: rule::ActiveModel = rule::Entity::find_by_id(id)
        .one(db)
        .await
        .expect("Error reading the rule!")
        .unwrap()
        .into();

    active_model.name = Set(name);
    active_model.label = Set(label);
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
