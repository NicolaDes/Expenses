use crate::database::{
    account::{self, Model as AccountModel},
    category,
    transaction::{self},
};
use askama::Template;
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Html,
    Json,
};
use chrono::{Datelike, NaiveDate, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};

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
    period_stats: PeriodStats,
    menu: &'a str,
    sub_menu: &'a str,
}

#[derive(Debug)]
pub struct PeriodStats {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Serialize)]
pub struct ChartData {
    montly_labels: Vec<String>,
    montly_expenses: Vec<f64>,
    montly_income: Vec<f64>,
    income_categories: Vec<String>,
    income_values: Vec<f64>,
    expense_categories_category: Vec<String>,
    expense_values_category: Vec<f64>,
    expense_categories_macrocategory: Vec<String>,
    expense_values_macrocategory: Vec<f64>,
    income: f64,
    expenses: f64,
    net_balance: f64,
    transactions_count: i32,
    transactions_count_used: i32,
    mean_montly_income: f64,
    mean_montly_expenses: f64,
}

#[derive(Deserialize)]
pub struct DateRange {
    start: String,
    end: String,
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

    let today = Utc::now().date_naive();
    let start_of_year = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();

    let html = AccountDetailTemplate {
        account: &account_model,
        transactions: transactions,
        period_stats: PeriodStats {
            start_date: start_of_year,
            end_date: today,
        },
        menu: "accounts",
        sub_menu: "detail",
    };

    Ok(Html(html.render().unwrap()))
}

pub async fn get_chart_data(
    Path(account_id): Path<i32>,
    Query(range): Query<DateRange>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<ChartData>, StatusCode> {
    // TODO: Replace with settigns reading - BEGIN
    // income-bonds-savings, outcome-bonds-savings, outcome-etf-investments,
    // income-etf-investments,income-refunds-refunds, outcome-investment-tax,
    // outcome-crypto-investments, income-crypto-investments
    let unused_category_ids = [2, 3, 4, 5, 21, 22, 23, 24];
    // TODO: Replace with settigns reading - END

    let mut montly_labels = vec![];
    let mut montly_expenses = vec![];
    let mut montly_income = vec![];
    let mut income_categories = vec![];
    let mut income_values = vec![];
    let mut expense_categories_category = vec![];
    let mut expense_values_category = vec![];
    let mut expense_categories_macrocategory = vec![];
    let mut expense_values_macrocategory = vec![];
    let mut income: f64 = 0.0;
    let mut expenses: f64 = 0.0;
    let mut transactions_count_used: i32 = 0;

    let start_date = chrono::NaiveDate::parse_from_str(&range.start, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let end_date = chrono::NaiveDate::parse_from_str(&range.end, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let transactions_count = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .filter(transaction::Column::Date.between(start_date, end_date))
        .count(&db)
        .await
        .unwrap() as i32;

    let transactions = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .filter(transaction::Column::CategoryId.is_not_in(unused_category_ids))
        .filter(transaction::Column::Date.between(start_date, end_date))
        .order_by_asc(transaction::Column::Date)
        .find_with_related(category::Entity)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

    for transaction_with_cat in transactions.unwrap() {
        transactions_count_used += 1;

        let montly_label = transaction_with_cat.0.date.format("%b %Y").to_string();
        let weighted_transaction_value = transaction_with_cat.0.value
            - (transaction_with_cat.0.value * (transaction_with_cat.0.perc_to_exclude as f64));

        if weighted_transaction_value > 0.0 {
            income += weighted_transaction_value;
        } else {
            expenses += weighted_transaction_value;
        }

        if !montly_labels.contains(&montly_label) {
            montly_labels.push(montly_label.clone());
            montly_expenses.push(0.0);
            montly_income.push(0.0);
        }

        let idx = montly_labels
            .iter()
            .position(|l| l == &montly_label)
            .unwrap();

        if weighted_transaction_value > 0.0 {
            montly_income[idx] += weighted_transaction_value;
        } else {
            montly_expenses[idx] += weighted_transaction_value;
        }

        if transaction_with_cat.0.value > 0.0 {
            let transaction_category = transaction_with_cat
                .1
                .into_iter()
                .next()
                .map(|c| c.category)
                .unwrap_or("N/A".to_string());

            if !income_categories.contains(&transaction_category) {
                income_categories.push(transaction_category.clone());
                income_values.push(0.0);
            }

            let income_idx = income_categories
                .iter()
                .position(|c| c == &transaction_category)
                .unwrap();

            income_values[income_idx] += weighted_transaction_value;
        } else {
            let transaction_category = transaction_with_cat
                .1
                .clone()
                .into_iter()
                .next()
                .map(|c| c.category)
                .unwrap_or("N/A".to_string());

            if !expense_categories_category.contains(&transaction_category) {
                expense_categories_category.push(transaction_category.clone());
                expense_values_category.push(0.0);
            }

            let expense_idx = expense_categories_category
                .iter()
                .position(|c| c == &transaction_category)
                .unwrap();

            expense_values_category[expense_idx] += weighted_transaction_value;

            let transaction_macrocategory = transaction_with_cat
                .1
                .into_iter()
                .next()
                .map(|c| c.macro_category)
                .unwrap_or("N/A".to_string());

            if !expense_categories_macrocategory.contains(&transaction_macrocategory) {
                expense_categories_macrocategory.push(transaction_macrocategory.clone());
                expense_values_macrocategory.push(0.0);
            }

            let expense_idx = expense_categories_macrocategory
                .iter()
                .position(|c| c == &transaction_macrocategory)
                .unwrap();

            expense_values_macrocategory[expense_idx] += weighted_transaction_value;
        }
    }

    let months_size = montly_labels.len() as f64;

    Ok(Json(ChartData {
        montly_labels,
        montly_expenses,
        montly_income,
        income_categories,
        income_values,
        expense_categories_category,
        expense_values_category,
        expense_categories_macrocategory,
        expense_values_macrocategory,
        income,
        expenses,
        net_balance: income + expenses,
        transactions_count,
        transactions_count_used,
        mean_montly_income: income / months_size,
        mean_montly_expenses: expenses / months_size,
    }))
}
