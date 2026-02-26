use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Extension, Form,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::database::{entities::label, labels};

#[derive(Template)]
#[template(path = "labels.html")]
struct LabelsTemplate<'a> {
    labels: Vec<label::Model>,
    menu: &'a str,
}

#[derive(Deserialize)]
pub struct AddLabelForm {
    name: String,
}

pub async fn get_labels_handler(
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Html<String>, StatusCode> {
    let all_labels = labels::get_all_labels(&db).await.map_err(|e| {
        eprintln!("Error retrieving labels: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let html = LabelsTemplate {
        labels: all_labels,
        menu: "labels",
    };

    Ok(Html(
        html.render()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn create_label_handler(
    Extension(db): Extension<DatabaseConnection>,
    Form(form): Form<AddLabelForm>,
) -> Result<Redirect, StatusCode> {
    labels::create_label(&db, form.name).await.map_err(|e| {
        eprintln!("Error creating label: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;
    Ok(Redirect::to("/labels"))
}

pub async fn delete_label_handler(
    Path(label_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    match labels::delete_label(&db, label_id).await {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(err) => (StatusCode::CONFLICT, err.to_string()).into_response(),
    }
}
