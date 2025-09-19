use askama::Template;
use axum::{extract::Path, http::StatusCode, response::Redirect, Extension, Form};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};

use crate::database::entities::{account, settings};

#[derive(Template)]
#[template(path = "account_settings.html")]
struct SettingsTemplate<'a> {
    account: account::Model,
    settings: settings::Model,
    menu: &'a str,
    sub_menu: &'a str,
}

#[derive(serde::Deserialize)]
pub struct UpdateSettingForm {
    date_index: i32,
    description_index: i32,
    value_index: i32,
    starter_string: String,
}

pub async fn get_account_setting_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let account_data = account::Entity::find_by_id(account_id)
        .one(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore nel recupero account: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let settings: settings::Model = match account_data.find_related(settings::Entity).one(&db).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            let new_setting = settings::ActiveModel {
                account_id: Set(account_data.id),
                date_index: Set(0),
                description_index: Set(0),
                value_index: Set(0),
                starter_string: Set("".to_string()),
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

    let html = SettingsTemplate {
        account: account_data,
        settings,
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
    the_settings.update(&db).await.map_err(|err| {
        println!("Cannot update settings: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Redirect::to(&format!("/accounts/{}/settings", account_id)))
}
