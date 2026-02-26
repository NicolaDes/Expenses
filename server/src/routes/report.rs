use std::io;

use axum::extract::Query;
use csv::WriterBuilder;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::{
    database::{settings, transaction},
    routes::common::DateRange,
};

pub async fn get_splittable_expenses_report(
    account_id: i32,
    Query(range): &Query<DateRange>,
    excluded_categories: Vec<i32>,
    db: &DatabaseConnection,
) -> io::Result<Vec<u8>> {
    let settings = settings::Entity::find()
        .filter(settings::Column::AccountId.eq(account_id))
        .one(db)
        .await
        .map_err(|e| {
            eprintln!("Cannot query settings: {:?}", e);
            io::Error::other("Database error")
        })?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Settings not found"))?;

    let delimiter: u8 = settings
        .report_delimiter
        .as_bytes()
        .first()
        .copied()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "report_delimiter is empty"))?;
    let decimal_separator = settings.report_decimal_separator.as_str();

    let start_date = chrono::NaiveDate::parse_from_str(&range.start, "%Y-%m-%d")
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid date"))?;
    let end_date = chrono::NaiveDate::parse_from_str(&range.end, "%Y-%m-%d")
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid date"))?;

    let mut writer = WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(vec![]);

    let transactions = transaction::Entity::find()
        .filter(transaction::Column::AccountId.eq(account_id))
        .filter(transaction::Column::Date.between(start_date, end_date))
        .filter(transaction::Column::CategoryId.is_not_in(excluded_categories))
        .filter(transaction::Column::PercToExclude.ne(0.0))
        .order_by_asc(transaction::Column::Date)
        .all(db)
        .await
        .map_err(|e| io::Error::other(format!("DB error: {e}")))?;

    writer.write_record(["Descrizione", "Speso Netto", "Da Pagare"])?;

    for transaction in transactions {
        let value = transaction.value;
        let value_str = format!("{:.2}", value).replace('.', decimal_separator);

        let weighted_value = transaction.value * transaction.perc_to_exclude as f64;
        let weighted_value_str = format!("{:.2}", weighted_value).replace('.', decimal_separator);

        let _ = writer.write_record(&[
            transaction.description.clone(),
            value_str,
            weighted_value_str,
        ]);
    }

    writer.flush()?;

    let inner = writer
        .into_inner()
        .map_err(io::Error::other)?;

    Ok(inner)
}
