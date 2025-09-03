use askama::Template;
use axum::{response::Redirect, Extension, Form};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};

use crate::database::entities::category;

#[derive(Template)]
#[template(path = "categories.html")]
struct CategoriesTemplate<'a> {
    categories: Vec<category::Model>,
    menu: &'a str,
}

#[derive(serde::Deserialize)]
pub struct AddCategoryForm {
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
