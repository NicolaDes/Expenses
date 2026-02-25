use askama::Template;
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DatabaseConnection, EntityTrait,
    Statement, TransactionTrait,
};
use serde::Serialize;
use serde_json::from_slice;

use crate::{
    database::{
        account_rule, budget, category, entities::account, rule, settings,
        settings_excluded_category, transaction,
    },
    routes::backup::{get_full_backup, FullBackupDTO},
};

#[derive(Template)]
#[template(path = "utilities.html")]
struct AccountRulesTemplate<'a> {
    menu: &'a str,
}

#[derive(Serialize)]
struct RestoreSummary {
    accounts: usize,
    categories: usize,
    rules: usize,
    budgets: usize,
    transactions: usize,
    account_rules: usize,
    settings: usize,
    excluded_categories: usize,
}

pub async fn reset_sequence(
    db: &DatabaseConnection,
    table: &str,
    sequence: &str,
) -> anyhow::Result<()> {
    let sql = format!(
        "SELECT setval('{}', (SELECT COALESCE(MAX(id), 0) FROM {}), true);",
        sequence, table
    );
    db.execute(Statement::from_string(db.get_database_backend(), sql))
        .await?;
    Ok(())
}

pub async fn get_utilities_handler(
    Extension(_db): Extension<DatabaseConnection>,
) -> Result<Html<String>, StatusCode> {
    let html = AccountRulesTemplate { menu: "utilities" };

    Ok(Html(html.render().unwrap()))
}

pub async fn get_backup_handler(Extension(db): Extension<DatabaseConnection>) -> Response {
    let json_backup = match get_full_backup(&db).await {
        Ok(json) => json,
        Err(status) => return status.into_response(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(
            "Content-Disposition",
            "attachment; filename=\"backup.json\"",
        )
        .header("Content-Type", "application/json")
        .body(json_backup.into())
        .unwrap()
}

pub async fn restore_full_backup(
    Extension(db): Extension<DatabaseConnection>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut backup: Option<FullBackupDTO> = None;
    let mut summary = RestoreSummary {
        accounts: 0,
        categories: 0,
        rules: 0,
        budgets: 0,
        transactions: 0,
        account_rules: 0,
        settings: 0,
        excluded_categories: 0,
    };

    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("backup_file") {
            let data = match field.bytes().await {
                Ok(d) => d,
                Err(err) => {
                    eprintln!("Errore leggendo file: {:?}", err);
                    return (StatusCode::BAD_REQUEST, "Errore leggendo il file").into_response();
                }
            };

            match from_slice::<FullBackupDTO>(&data) {
                Ok(parsed) => backup = Some(parsed),
                Err(err) => {
                    eprintln!("Errore deserializzando JSON: {:?}", err);
                    return (StatusCode::BAD_REQUEST, "Backup non valido").into_response();
                }
            }
        }
    }

    let Some(backup) = backup else {
        return (StatusCode::BAD_REQUEST, "Nessun file caricato").into_response();
    };

    let txn = match db.begin().await {
        Ok(t) => t,
        Err(err) => {
            eprintln!("Errore avviando transazione: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Errore avviando transazione",
            )
                .into_response();
        }
    };

    macro_rules! rollback_on_err {
        ($result:expr, $msg:expr) => {
            match $result {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{}: {:?}", $msg, err);
                    if let Err(rb_err) = txn.rollback().await {
                        eprintln!("Rollback failed after error: {:?}", rb_err);
                    }
                    return (StatusCode::INTERNAL_SERVER_ERROR, $msg).into_response();
                }
            }
        };
    }

    rollback_on_err!(
        account::Entity::delete_many().exec(&txn).await,
        "Errore eliminando accounts"
    );
    rollback_on_err!(
        category::Entity::delete_many().exec(&txn).await,
        "Errore eliminando categorie"
    );
    rollback_on_err!(
        transaction::Entity::delete_many().exec(&txn).await,
        "Errore eliminando transazioni"
    );
    rollback_on_err!(
        budget::Entity::delete_many().exec(&txn).await,
        "Errore eliminando budgets"
    );
    rollback_on_err!(
        account_rule::Entity::delete_many().exec(&txn).await,
        "Errore eliminando account_rules"
    );
    rollback_on_err!(
        rule::Entity::delete_many().exec(&txn).await,
        "Errore eliminando rules"
    );
    rollback_on_err!(
        settings::Entity::delete_many().exec(&txn).await,
        "Errore eliminando settings"
    );
    rollback_on_err!(
        settings_excluded_category::Entity::delete_many()
            .exec(&txn)
            .await,
        "Errore eliminando excluded categories"
    );

    for a in backup.accounts {
        rollback_on_err!(
            account::ActiveModel {
                id: Set(a.id),
                name: Set(a.name),
            }
            .insert(&txn)
            .await,
            "Errore inserendo account"
        );
        summary.accounts += 1;
    }

    for c in backup.categories {
        rollback_on_err!(
            category::ActiveModel {
                id: Set(c.id),
                transaction_type: Set(c.transaction_type),
                macro_category: Set(c.macro_category),
                category: Set(c.category),
            }
            .insert(&txn)
            .await,
            "Errore inserendo categoria"
        );
        summary.categories += 1;
    }

    for t in backup.transactions {
        rollback_on_err!(
            transaction::ActiveModel {
                id: Set(t.id),
                account_id: Set(t.account_id),
                category_id: Set(t.category_id),
                value: Set(t.value),
                description: Set(t.description),
                date: Set(t.date.naive_utc()),
                perc_to_exclude: Set(t.perc_to_exclude),
                label: Set(t.label),
            }
            .insert(&txn)
            .await,
            "Errore inserendo transazione"
        );
        summary.transactions += 1;
    }

    for b in backup.budgets {
        rollback_on_err!(
            budget::ActiveModel {
                id: Set(b.id),
                account_id: Set(b.account_id),
                name: Set(b.name),
                value: Set(b.value),
            }
            .insert(&txn)
            .await,
            "Errore inserendo budget"
        );
        summary.budgets += 1;
    }

    for r in backup.rules {
        rollback_on_err!(
            rule::ActiveModel {
                id: Set(r.id),
                name: Set(r.name),
                label: Set(r.label),
                percentage: Set(r.percentage),
                category_id: Set(r.category_id),
                regexpr: Set(r.regexpr),
                date_start: Set(r.date_start),
                date_end: Set(r.date_end),
            }
            .insert(&txn)
            .await,
            "Errore inserendo rule"
        );
        summary.rules += 1;
    }

    for ar in backup.account_rules {
        rollback_on_err!(
            account_rule::ActiveModel {
                id: Set(ar.id),
                account_id: Set(ar.account_id),
                rule_id: Set(ar.rule_id),
            }
            .insert(&txn)
            .await,
            "Errore inserendo account_rule"
        );
        summary.account_rules += 1;
    }

    for s in backup.settings {
        rollback_on_err!(
            settings::ActiveModel {
                id: Set(s.id),
                account_id: Set(s.account_id),
                date_index: Set(s.date_index),
                description_index: Set(s.description_index),
                value_index: Set(s.value_index),
                starter_string: Set(s.starter_string),
                report_delimiter: Set(s.report_delimiter),
                report_decimal_separator: Set(s.report_decimal_separator),
            }
            .insert(&txn)
            .await,
            "Errore inserendo settings"
        );
        summary.settings += 1;
    }

    for ec in backup.exlcuded_categories {
        rollback_on_err!(
            settings_excluded_category::ActiveModel {
                id: Set(ec.id),
                settings_id: Set(ec.settings_id),
                category_id: Set(ec.category_id),
            }
            .insert(&txn)
            .await,
            "Errore inserendo excluded category"
        );
        summary.excluded_categories += 1;
    }

    if let Err(err) = txn.commit().await {
        eprintln!("Errore committando transazione: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Errore committando il ripristino",
        )
            .into_response();
    }

    // Reset sequences after commit, using the main connection
    let sequences = [
        ("accounts", "accounts_id_seq"),
        ("categories", "categories_id_seq"),
        ("transactions", "transactions_id_seq"),
        ("budgets", "budgets_id_seq"),
        ("rules", "rules_id_seq"),
        ("account_rules", "account_rules_id_seq"),
        ("settings", "settings_id_seq"),
        (
            "settings_excluded_categories",
            "settings_excluded_categories_id_seq",
        ),
    ];

    for (table, seq) in sequences.iter() {
        if let Err(e) = reset_sequence(&db, table, seq).await {
            eprintln!("Errore riallineando sequenza {}: {:?}", seq, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Errore riallineando i seriali",
            )
                .into_response();
        }
    }

    (StatusCode::OK, axum::Json(summary)).into_response()
}
