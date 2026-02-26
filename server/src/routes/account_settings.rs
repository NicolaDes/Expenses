use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Extension, Form,
};
use sea_orm::DatabaseConnection;

use crate::database::{
    accounts, categories, settingss,
    category,
    entities::{account, settings},
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
    let account_data = accounts::get_account(&db, account_id).await.map_err(|e| {
        eprintln!("Error retrieving account: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let settings = settingss::get_settings_for_account(&db, account_id)
        .await
        .map_err(|e| {
            eprintln!("Error retrieving settings: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let cats = categories::get_categories(&db).await.map_err(|e| {
        eprintln!("Error retrieving categories: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let excluded_categories = settingss::get_excluded_categories_for_account(&db, account_id)
        .await
        .map_err(|e| {
            eprintln!("Error retrieving excluded categories: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let html = SettingsTemplate {
        account: account_data,
        settings,
        categories: cats,
        excluded_categories,
        menu: "accounts",
        sub_menu: "settings",
    };

    let rendered = html.render().map_err(|e| {
        eprintln!("Template render error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(axum::response::Html(rendered))
}

pub async fn update_setting_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<UpdateSettingForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    settingss::update_settings(
        &db,
        account_id,
        form.date_index,
        form.description_index,
        form.value_index,
        form.starter_string,
        form.report_delimiter,
        form.report_decimal_separator,
    )
    .await
    .map_err(|e| {
        eprintln!("Error updating settings: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Redirect::to(&format!("/accounts/{}/settings", account_id)))
}

pub async fn add_excluded_category_handler(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddExcludedCategoryForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    settingss::add_excluded_category(&db, account_id, form.category_id)
        .await
        .map_err(|e| {
            eprintln!("Error adding excluded category: {:?}", e);
            axum::http::StatusCode::BAD_REQUEST
        })?;

    Ok(Redirect::to(&format!("/accounts/{}/settings", account_id)))
}

pub async fn delete_excluded_category_handler(
    Path((account_id, category_id)): Path<(i32, i32)>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match settingss::delete_excluded_category(&db, account_id, category_id).await {
        Ok(_) => axum::http::StatusCode::NO_CONTENT,
        Err(err) => {
            eprintln!("Error deleting excluded category: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
