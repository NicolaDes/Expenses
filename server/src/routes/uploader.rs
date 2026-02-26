use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use calamine::{Reader, Xls, Xlsx};
use chrono::{Duration, NaiveDate};
use sea_orm::DatabaseConnection;
use serde::Serialize;
use std::io::Cursor;

use crate::database::{settingss, transactions};

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
    _data: &[u8],
    _date_idx: usize,
    _description_idx: usize,
    _value_idx: usize,
    _starter_string: String,
) -> anyhow::Result<Vec<TransactionData>> {
    anyhow::bail!("CSV import is not yet supported")
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

            let raw_date = values
                .get(date_idx)
                .ok_or_else(|| anyhow::anyhow!("date column index {} out of range (row has {} cols)", date_idx, values.len()))?;
            let date = excel_number_to_date(raw_date)
                .ok_or_else(|| anyhow::anyhow!("cannot parse date value {:?}", raw_date))?;

            let description = values
                .get(description_idx)
                .ok_or_else(|| anyhow::anyhow!("description column index {} out of range", description_idx))?
                .clone();

            let value: f64 = values
                .get(value_idx)
                .ok_or_else(|| anyhow::anyhow!("value column index {} out of range", value_idx))?
                .replace(',', ".")
                .parse()
                .map_err(|_| anyhow::anyhow!("value column is not a valid number"))?;

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

            let raw_date = values
                .get(date_idx)
                .ok_or_else(|| anyhow::anyhow!("date column index {} out of range (row has {} cols)", date_idx, values.len()))?;
            let date = excel_number_to_date(raw_date)
                .ok_or_else(|| anyhow::anyhow!("cannot parse date value {:?}", raw_date))?;

            let description = values
                .get(description_idx)
                .ok_or_else(|| anyhow::anyhow!("description column index {} out of range", description_idx))?
                .clone();

            let value: f64 = values
                .get(value_idx)
                .ok_or_else(|| anyhow::anyhow!("value column index {} out of range", value_idx))?
                .replace(',', ".")
                .parse()
                .map_err(|_| anyhow::anyhow!("value column is not a valid number"))?;

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
    let mut transaction_data = Vec::new();
    let mut processed_transactions = 0;

    let settings = match settingss::get_settings_for_account(&db, account_id).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error retrieving settings: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error retrieving settings").into_response();
        }
    };

    let date_index: usize = settings.date_index as usize;
    let description_index: usize = settings.description_index as usize;
    let value_index: usize = settings.value_index as usize;
    let starter_string = settings.starter_string.clone();

    while let Some(field) = match multipart.next_field().await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error reading multipart field: {:?}", e);
            return (StatusCode::BAD_REQUEST, "Error reading uploaded file").into_response();
        }
    } {
        let filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "file".to_string());

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error reading file bytes: {:?}", e);
                return (StatusCode::BAD_REQUEST, "Error reading file bytes").into_response();
            }
        };

        let parsed = if filename.ends_with(".csv") {
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

        match parsed {
            Ok(new_txts) => transaction_data.extend(new_txts),
            Err(e) => {
                eprintln!("Errore import file {}: {:?}", filename, e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Errore import file").into_response();
            }
        }
    }

    for tx in transaction_data {
        let naive_dt: chrono::NaiveDateTime = tx.date.into();
        if let Err(e) = transactions::create_transaction(
            &db,
            account_id,
            None,
            tx.value,
            tx.description,
            naive_dt,
            0.0,
            "".to_owned(),
        )
        .await
        {
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
