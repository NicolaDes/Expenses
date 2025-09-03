use crate::database::{
    account::{self, Model as AccountModel},
    category,
    transaction::{self},
};
use askama::Template;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::Html,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

#[derive(Debug)]
struct TransactionWithCategory {
    txt: transaction::Model,
    category_name: String,
}

#[derive(Template)]
#[template(path = "account_detail.html")]
struct AccountDetailTemplate<'a> {
    account: &'a AccountModel,
    transactions: Vec<TransactionWithCategory>,
    total_income: f64,
    total_expense: f64,
    menu: &'a str,
    sub_menu: &'a str,
}

pub async fn get_account_detail(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Html<String>, StatusCode> {
    let account_model = account::Entity::find_by_id(account_id)
        .one(&db)
        .await
        .expect("Errore DB")
        .expect("Account non trovato");

    let txs_with_cats = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .find_with_related(category::Entity)
        .all(&db)
        .await
        .map_err(|e| {
            eprintln!("Errore find_with_related: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let transactions: Vec<TransactionWithCategory> = txs_with_cats
        .into_iter()
        .map(|(txt, cats)| {
            let category_name = cats
                .into_iter()
                .next()
                .map(|c| c.category)
                .unwrap_or_else(|| "-".to_string());

            TransactionWithCategory { txt, category_name }
        })
        .collect();

    let mut total_income = 0.0;
    let mut total_expense = 0.0;
    for t in &transactions {
        if t.txt.value >= 0.0 {
            total_income += t.txt.value;
        } else {
            total_expense += t.txt.value.abs();
        }
    }

    let html = AccountDetailTemplate {
        account: &account_model,
        transactions: transactions,
        total_income,
        total_expense,
        menu: "accounts",
        sub_menu: "detail",
    };

    Ok(Html(html.render().unwrap()))
}

// pub async fn upload_transactions(
//     account_id: i32,
//     Extension(db): Extension<DatabaseConnection>,
//     mut multipart: Multipart,
// ) -> impl IntoResponse {
//     while let Some(field) = multipart.next_field().await.unwrap() {
//         let name = field.name().unwrap_or("").to_string();
//         if name == "file" {
//             let data = field.bytes().await.unwrap();
//         }
//     }

//     let url = format!("/accounts/{}", account_id);
//     Redirect::to(&url).into_response()
// }
