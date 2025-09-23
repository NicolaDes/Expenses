use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Extension, Form,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use serde::Deserialize;

use crate::database::entities::category;

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
    let categories = match category::Entity::find().all(&db).await {
        Ok(cats) => cats,
        Err(e) => {
            println!("Errore find categories: {:?}", e);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let html = CategoriesTemplate {
        categories,
        menu: "categories",
    };
    Ok(axum::response::Html(html.render().unwrap()))
}

pub async fn add_category_handler(
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddCategoryForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    let new_category = category::ActiveModel {
        transaction_type: Set(form.transaction_type),
        macro_category: Set(form.macro_category),
        category: Set(form.category),
        ..Default::default()
    };

    if let Err(e) = new_category.insert(&db).await {
        eprintln!("Errore inserimento category: {:?}", e);
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }
    Ok(Redirect::to(&format!("/categories",)))
}

pub async fn delete_category(
    Path(category_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match category::Entity::delete_by_id(category_id).exec(&db).await {
        Ok(_) => axum::http::StatusCode::NO_CONTENT,
        Err(err) => {
            eprintln!("Errore eliminando transazione {}: {}", category_id, err);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn edit_category(
    Path(category_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<CategoryForm>,
) -> impl IntoResponse {
    let mut category: category::ActiveModel = category::Entity::find_by_id(category_id)
        .one(&db)
        .await
        .expect("Error reading the category!")
        .unwrap()
        .into();

    category.transaction_type = Set(form.transaction_type);
    category.macro_category = Set(form.macro_category);
    category.category = Set(form.category);

    let _ = category.update(&db).await.map_err(|err| {
        eprintln!("Cannot update category: {}", err);
        return StatusCode::INTERNAL_SERVER_ERROR;
    });

    return StatusCode::OK;
}
