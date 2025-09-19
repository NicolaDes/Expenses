use askama::Template;
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use serde_json::from_slice;

use crate::{
    database::{account_rule, budget, category, entities::account, rule, transaction},
    routes::backup::{get_full_backup, FullBackupDTO},
};

#[derive(Template)]
#[template(path = "settings.html")]
struct AccountRulesTemplate<'a> {
    menu: &'a str,
}

pub async fn get_account_settings_handler(
    Extension(_db): Extension<DatabaseConnection>,
) -> Result<Html<String>, StatusCode> {
    let html = AccountRulesTemplate { menu: "accounts" };

    Ok(Html(html.render().unwrap()))
}

pub async fn get_backup_account_handler(Extension(db): Extension<DatabaseConnection>) -> Response {
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

    if let Err(err) = account::Entity::delete_many().exec(&db).await {
        eprintln!("Errore cancellando accounts: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Errore eliminando transazioni",
        )
            .into_response();
    }

    if let Err(err) = transaction::Entity::delete_many().exec(&db).await {
        eprintln!("Errore cancellando transazioni: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Errore eliminando transazioni",
        )
            .into_response();
    }

    if let Err(err) = budget::Entity::delete_many().exec(&db).await {
        eprintln!("Errore cancellando budgets: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Errore eliminando budgets",
        )
            .into_response();
    }

    if let Err(err) = account_rule::Entity::delete_many().exec(&db).await {
        eprintln!("Errore cancellando account_rules: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Errore eliminando account_rules",
        )
            .into_response();
    }

    for a in backup.accounts {
        let _ = account::ActiveModel {
            id: Set(a.id),
            name: Set(a.name),
            balance: Set(a.balance),
        }
        .insert(&db)
        .await;
    }

    for t in backup.transactions {
        let _ = transaction::ActiveModel {
            id: Set(t.id),
            account_id: Set(t.account_id),
            category_id: Set(t.category_id),
            value: Set(t.value),
            description: Set(t.description),
            date: Set(t.date.naive_utc()),
            perc_to_exclude: Set(t.perc_to_exclude),
            label: Set(t.label),
        }
        .insert(&db)
        .await;
    }

    for b in backup.budgets {
        let _ = budget::ActiveModel {
            id: Set(b.id),
            account_id: Set(b.account_id),
            name: Set(b.name),
            value: Set(b.value),
        }
        .insert(&db)
        .await;
    }

    for ar in backup.account_rules {
        let _ = account_rule::ActiveModel {
            id: Set(ar.id),
            account_id: Set(ar.account_id),
            rule_id: Set(ar.rule_id),
        }
        .insert(&db)
        .await;
    }

    for r in backup.rules {
        let _ = rule::ActiveModel {
            id: Set(r.id),
            name: Set(r.name),
            label: Set(r.label),
            percentage: Set(r.percentage),
            category_id: Set(r.category_id),
            regexpr: Set(r.regexpr),
            date_start: Set(r.date_start),
            date_end: Set(r.date_end),
        }
        .insert(&db)
        .await;
    }

    for c in backup.categories {
        let _ = category::ActiveModel {
            id: Set(c.id),
            transaction_type: Set(c.transaction_type),
            macro_category: Set(c.macro_category),
            category: Set(c.category),
        }
        .insert(&db)
        .await;
    }

    (StatusCode::OK, "Backup ripristinato con successo").into_response()
}
