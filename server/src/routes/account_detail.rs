use crate::{
    database::{
        account::{self, Model as AccountModel},
        budget, category,
        transaction::{self},
    },
    routes::{common::DateRange, report::get_splittable_expenses_report},
};
use askama::Template;
use axum::{
    extract::{Extension, Path, Query},
    http::{header, Response, StatusCode},
    response::{Html, IntoResponse},
    Json,
};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde::Serialize;

#[derive(Template)]
#[template(path = "account_detail.html")]
struct AccountDetailTemplate<'a> {
    account: &'a AccountModel,
    period_stats: PeriodStats,
    budgets: Vec<BudgetsTemplate>,
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
    mean_income_increment: f64,
    mean_income_increment_percentage: f64,
    mean_expenses_increment: f64,
    mean_expenses_increment_percentage: f64,
    mean_montly_net_balance: f64,
    mean_net_balance_increment: f64,
    mean_net_balance_increment_percentage: f64,
}

#[derive(Serialize)]
pub struct BudgetsTemplate {
    label: String,
    value: f64,
    limit: f64,
    percentage: i32,
    year: i32,
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

    let today = Utc::now().date_naive();
    let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    let last_day_prev_month = first_of_month - Duration::days(1);
    let start_of_year = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();

    let budget_models = budget::Entity::find()
        .filter(budget::Column::AccountId.eq(account_id))
        .all(&db)
        .await
        .unwrap();

    let mut budgets: Vec<BudgetsTemplate> = vec![];

    for budget_model in budget_models {
        let mut sum = 0.0;

        let category_option = category::Entity::find()
            .filter(category::Column::Category.eq(budget_model.name.clone()))
            .one(&db)
            .await
            .unwrap();

        if let Some(category_model) = category_option {
            let transactions = transaction::Entity::find()
                .filter(transaction::Column::AccountId.eq(account_id))
                .filter(transaction::Column::Date.gt(start_of_year))
                .filter(transaction::Column::CategoryId.eq(category_model.id))
                .all(&db)
                .await
                .unwrap();

            for transaction in transactions {
                sum = sum
                    + (transaction.value
                        - (transaction.value * transaction.perc_to_exclude as f64))
                        .abs();
            }
        }

        budgets.push(BudgetsTemplate {
            label: budget_model.name.clone(),
            value: sum,
            limit: budget_model.value,
            percentage: ((sum / budget_model.value) * 100.0 as f64) as i32,
            year: start_of_year.year(),
        });
    }

    let html = AccountDetailTemplate {
        account: &account_model,
        period_stats: PeriodStats {
            start_date: start_of_year,
            end_date: last_day_prev_month,
        },
        budgets,
        menu: "accounts",
        sub_menu: "detail",
    };

    Ok(Html(html.render().unwrap()))
}

pub async fn get_expenses_report(
    Path(account_id): Path<i32>,
    Query(range): Query<DateRange>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    // TODO: Replace with settigns reading - BEGIN
    let excluded_category_ids: Vec<i32> = vec![1, 2, 3, 4, 5, 21, 22, 23, 24];
    // TODO: Replace with settigns reading - END

    let data =
        match get_splittable_expenses_report(account_id, &Query(range), excluded_category_ids, &db)
            .await
        {
            Ok(csv) => csv,
            Err(_) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate CSV")
                    .into_response()
            }
        };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/csv")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"report.csv\"",
        )
        .body(data.into())
        .unwrap()
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

    let net_balance = income + expenses;
    let net_balance_vec: Vec<f64> = montly_income
        .iter()
        .zip(montly_expenses.iter())
        .map(|(x, y)| x + y)
        .collect();

    let income_except_last_month = montly_income[..montly_income.len() - 1].iter().sum::<f64>();
    let mean_income_increment =
        (income_except_last_month / (months_size - 1 as f64)) - (income / months_size);
    let mean_income_increment_percentage = (((income / months_size)
        - (income_except_last_month / (months_size - 1 as f64)))
        / (income_except_last_month / (months_size - 1 as f64)))
        * 100 as f64;

    let expenses_except_last_month = montly_expenses[..montly_expenses.len() - 1]
        .iter()
        .sum::<f64>();
    let mean_expenses_increment =
        (expenses_except_last_month / (months_size - 1 as f64)) - (expenses / months_size);
    let mean_expenses_increment_percentage = (((expenses / months_size)
        - (expenses_except_last_month / (months_size - 1 as f64)))
        / (expenses_except_last_month / (months_size - 1 as f64)))
        * 100 as f64;

    let net_balance_except_last_month = net_balance_vec[..net_balance_vec.len() - 1]
        .iter()
        .sum::<f64>();
    let mean_montly_net_balance = net_balance / months_size;
    let mean_net_balance_increment =
        (net_balance_except_last_month / (months_size - 1 as f64)) - (net_balance / months_size);
    let mean_net_balance_increment_percentage = (((net_balance / months_size)
        - (net_balance_except_last_month / (months_size - 1 as f64)))
        / (net_balance_except_last_month / (months_size - 1 as f64)))
        * 100 as f64;

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
        net_balance,
        transactions_count,
        transactions_count_used,
        mean_montly_income: income / months_size,
        mean_montly_expenses: expenses / months_size,
        mean_income_increment,
        mean_income_increment_percentage,
        mean_expenses_increment,
        mean_expenses_increment_percentage,
        mean_montly_net_balance,
        mean_net_balance_increment,
        mean_net_balance_increment_percentage,
    }))
}
