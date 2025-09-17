use askama::Template;
use axum::Extension;
use sea_orm::DatabaseConnection;

#[derive(Template)]
#[template(path = "index.html")]
struct RulesTemplate<'a> {
    menu: &'a str,
}

pub async fn get_index(
    Extension(_db): Extension<DatabaseConnection>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let html = RulesTemplate { menu: "" };

    Ok(axum::response::Html(html.render().unwrap()))
}
