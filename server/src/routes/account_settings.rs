use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Extension, Form,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};

use crate::database::{
    category,
    entities::{account, settings},
    settings_excluded_category,
};

#[derive(Template)]
#[template(path = "account_settings.html")]
struct SettingsTemplate<'a> {
    account: account::Model,
    settings: settings::Model,
    categories: Vec<category::Model>,
    excluded_categories: Vec<category::Model>,
    menu: &'a str,
    sub_menu: &'a str,
}

#[derive(serde::Deserialize)]
pub struct UpdateSettingForm {
    date_index: i32,
    description_index: i32,
    value_index: i32,
    starter_string: String,
    report_delimiter: String,
    report_decimal_separator: String,
}

#[derive(serde::Deserialize)]
pub struct AddExcludedCategoryForm {
    category_id: i32,
}

pub async fn get_account_setting_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    // TODO: Move into database modules
    let account_data = account::Entity::find_by_id(account_id)
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore nel recupero account: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // TODO: Move into database modules
    let settings: settings::Model = match account_data.find_related(settings::Entity).one(&db).await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            let new_setting = settings::ActiveModel {
                account_id: Set(account_data.id),
                date_index: Set(0),
                description_index: Set(0),
                value_index: Set(0),
                starter_string: Set("".to_string()),
                report_delimiter: Set(";".to_string()),
                report_decimal_separator: Set(",".to_string()),
                ..Default::default()
            };

            let inserted = new_setting.insert(&db).await.map_err(|e| {
                eprintln!("Errore creazione setting di default: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            inserted
        }
        Err(e) => {
            eprintln!("Errore recupero setting: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let categories = category::Entity::find().all(&db).await.map_err(|e| {
        eprint!("Error retrieving categories: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // TODO: Move into database modules
    let excluded_categories = settings
        .find_related(category::Entity)
        .all(&db)
        .await
        .map_err(|e| {
            eprint!(
                "Error retrieving excluded categories from settings: {:?}",
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let html = SettingsTemplate {
        account: account_data,
        settings,
        categories,
        excluded_categories,
        menu: "accounts",
        sub_menu: "settings",
    };

    Ok(axum::response::Html(html.render().unwrap()))
}

pub async fn update_setting_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<UpdateSettingForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    // TODO: Move into database modules
    let settings: settings::Model = settings::Entity::find()
        .filter(settings::Column::AccountId.eq(account_id))
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore query settings: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut the_settings: settings::ActiveModel = settings.into();
    the_settings.date_index = Set(form.date_index);
    the_settings.description_index = Set(form.description_index);
    the_settings.value_index = Set(form.value_index);
    the_settings.starter_string = Set(form.starter_string);
    the_settings.report_delimiter = Set(form.report_delimiter);
    the_settings.report_decimal_separator = Set(form.report_decimal_separator);

    // TODO: Move into database modules
    the_settings.update(&db).await.map_err(|err| {
        println!("Cannot update settings: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Redirect::to(&format!("/accounts/{}/settings", account_id)))
}

pub async fn add_excluded_category_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddExcludedCategoryForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    // TODO: Move into database modules
    let settings: settings::Model = settings::Entity::find()
        .filter(settings::Column::AccountId.eq(account_id))
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore query settings: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let the_settings_excluded_category = settings_excluded_category::ActiveModel {
        settings_id: Set(settings.id),
        category_id: Set(form.category_id),
        ..Default::default()
    };

    // TODO: Move into database modules
    if let Err(e) = the_settings_excluded_category.insert(&db).await {
        eprintln!("Error inserting the excluded category in settings: {:?}", e);
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    Ok(Redirect::to(&format!("/accounts/{}/settings", account_id)))
}

pub async fn delete_excluded_category_handler(
    Path((account_id, category_id)): Path<(i32, i32)>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    // TODO: Move into database modules
    let settings: settings::Model = settings::Entity::find()
        .filter(settings::Column::AccountId.eq(account_id))
        .one(&db)
        .await
        .expect("Cannot find settings!")
        .unwrap()
        .into();

    // TODO: Move into database modules
    let the_settings_excluded_category: settings_excluded_category::Model =
        settings_excluded_category::Entity::find()
            .filter(settings_excluded_category::Column::SettingsId.eq(settings.id))
            .filter(settings_excluded_category::Column::CategoryId.eq(category_id))
            .one(&db)
            .await
            .expect("Cannot find the settings excluded category!")
            .unwrap()
            .into();

    // TODO: Move into database modules
    match settings_excluded_category::Entity::delete_by_id(the_settings_excluded_category.id)
        .exec(&db)
        .await
    {
        Ok(_) => axum::http::StatusCode::NO_CONTENT,
        Err(err) => {
            eprintln!("Cannot delete settings_excluded_category: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
