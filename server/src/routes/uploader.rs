use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use calamine::{Reader, Xls, Xlsx};
use chrono::{Duration, NaiveDate};
use csv::ReaderBuilder;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::Serialize;
use std::io::Cursor;

use crate::database::{settings, transaction};

#[derive(Serialize)]
struct ImportSummary {
    rows_imported: usize,
}

struct TransactionData {
    description: String,
    value: f64,
    date: NaiveDate,
}

fn excel_number_to_date(excel_number: &str) -> Option<NaiveDate> {
    let n: i64 = excel_number.parse().ok()?;
    let base_date = NaiveDate::from_ymd_opt(1900, 1, 1)?;
    Some(base_date + Duration::days(n - 2))
}

async fn process_csv(
    data: &[u8],
    _date_idx: usize,
    _description_idx: usize,
    _value_idx: usize,
    _starter_string: String,
) -> anyhow::Result<Vec<TransactionData>> {
    let transactions = Vec::new();

    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(Cursor::new(data));

    for (_idx, _result) in rdr.records().enumerate() {
        // let record = result?;
        todo!();
    }

    Ok(transactions)
}

async fn process_xlsx(
    data: &[u8],
    date_idx: usize,
    description_idx: usize,
    value_idx: usize,
    starter_string: String,
) -> anyhow::Result<Vec<TransactionData>> {
    let mut transactions = Vec::new();
    let cursor = Cursor::new(data);
    let mut workbook: Xlsx<_> = Xlsx::new(cursor)?;

    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        let mut found = false;
        for row in range.rows() {
            let values: Vec<String> = row.iter().map(|c| c.to_string()).collect();

            if !found {
                if values.iter().any(|v| v.contains(&starter_string)) {
                    found = true;
                    continue;
                } else {
                    continue;
                }
            }

            let date = excel_number_to_date(&values[date_idx]).unwrap();
            let description = values[description_idx].clone();
            let value: f64 = values[value_idx]
                .replace(',', ".")
                .parse()
                .expect("Not a Number");

            transactions.push(TransactionData {
                description: description,
                value: value,
                date: date,
            });
        }
    }

    Ok(transactions)
}

async fn process_xls(
    data: &[u8],
    date_idx: usize,
    description_idx: usize,
    value_idx: usize,
    starter_string: String,
) -> anyhow::Result<Vec<TransactionData>> {
    let mut transactions = Vec::new();
    let cursor = Cursor::new(data);
    let mut workbook: Xls<_> = Xls::new(cursor)?;

    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        let mut found = false;
        for row in range.rows() {
            let values: Vec<String> = row.iter().map(|c| c.to_string()).collect();

            if !found {
                if values.iter().any(|v| v.contains(&starter_string)) {
                    found = true;
                    continue;
                } else {
                    continue;
                }
            }

            let date = excel_number_to_date(&values[date_idx]).unwrap();
            let description = values[description_idx].clone();
            let value: f64 = values[value_idx]
                .replace(',', ".")
                .parse()
                .expect("Not a Number");

            transactions.push(TransactionData {
                description: description,
                value: value,
                date: date,
            });
        }
    }

    Ok(transactions)
}

pub async fn upload_transaction_file(
    Path(account_id): Path<i32>,
    Extension(db): Extension<DatabaseConnection>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut transactions = Vec::new();
    let mut processed_transactions = 0;

    let settings = settings::Entity::find()
        .filter(settings::Column::AccountId.eq(account_id))
        .one(&db)
        .await
        .expect("Errore nel recupero di settings!")
        .unwrap();

    let date_index: usize = settings.date_index as usize;
    let description_index: usize = settings.description_index as usize;
    let value_index: usize = settings.value_index as usize;
    let starter_string: &String = &settings.starter_string;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "file".to_string());

        let data = field.bytes().await.unwrap();

        let parsed_transactions = if filename.ends_with(".csv") {
            process_csv(
                &data,
                date_index,
                description_index,
                value_index,
                starter_string.clone(),
            )
            .await
        } else if filename.ends_with(".xlsx") {
            process_xlsx(
                &data,
                date_index,
                description_index,
                value_index,
                starter_string.clone(),
            )
            .await
        } else if filename.ends_with(".xls") {
            process_xls(
                &data,
                date_index,
                description_index,
                value_index,
                starter_string.clone(),
            )
            .await
        } else {
            return (StatusCode::BAD_REQUEST, "Formato non supportato").into_response();
        };

        match parsed_transactions {
            Ok(new_txts) => transactions.extend(new_txts),
            Err(e) => {
                eprintln!("Errore import file {}: {:?}", filename, e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Errore import file").into_response();
            }
        }
    }

    for transaction in transactions {
        let model = transaction::ActiveModel {
            account_id: Set(account_id),
            description: Set(transaction.description),
            value: Set(transaction.value),
            date: Set(transaction.date.into()),
            perc_to_exclude: Set(0.0),
            label: Set("".to_owned()),
            ..Default::default()
        };

        if let Err(e) = model.insert(&db).await {
            eprintln!("Errore nell'inserimento della transazione: {:?}", e);
            continue;
        }
        processed_transactions += 1;
    }

    let summary = ImportSummary {
        rows_imported: processed_transactions,
    };
    (StatusCode::OK, Json(summary)).into_response()
}
