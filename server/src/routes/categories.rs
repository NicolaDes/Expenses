use askama::Template;
use axum::{
    extract::Path,
    response::{IntoResponse, Redirect},
    Extension, Form,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::database::{categories, entities::category};

#[derive(Template)]
#[template(path = "categories.html")]
struct CategoriesTemplate<'a> {
    categories: Vec<category::Model>,
    menu: &'a str,
}

#[derive(Deserialize)]
pub struct AddCategoryForm {
    transaction_type: String,
    macro_category: String,
    category: String,
}

#[derive(Deserialize)]
pub struct CategoryForm {
    transaction_type: String,
    macro_category: String,
    category: String,
}

pub async fn get_categories_handler(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let categories = match categories::get_categories(&db).await {
        Ok(cats) => cats,
        Err(_) => {
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let html = CategoriesTemplate {
        categories,
        menu: "categories",
    };
    Ok(axum::response::Html(
        html.render()
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn add_category_handler(
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddCategoryForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    match categories::create_category(
        &db,
        form.category,
        form.macro_category,
        form.transaction_type,
    )
    .await
    {
        Ok(_) => Ok(Redirect::to("/categories")),
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}

pub async fn delete_category(
    Path(category_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match categories::delete_category(&db, category_id).await {
        Ok(_) => (axum::http::StatusCode::NO_CONTENT).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

pub async fn edit_category(
    Path(category_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<CategoryForm>,
) -> impl IntoResponse {
    match categories::edit_category(
        &db,
        category_id,
        form.category,
        form.macro_category,
        form.transaction_type,
    )
    .await
    {
        Ok(_) => (axum::http::StatusCode::OK).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}
