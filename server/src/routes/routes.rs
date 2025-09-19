use axum::{
    response::Redirect,
    routing::{delete, get, post},
    Router,
};
use tower_http::services::ServeDir;

use crate::routes::{
    account_budgets::{add_budget_handler, get_account_budgets_handler},
    account_detail::get_account_detail,
    account_rules::{
        activate_rule_handler, add_account_rule_handler, apply_rules, deactivate_rule_handler,
        get_account_rules_handler, preview_apply_rules, resolve_conflicts_rules,
    },
    account_transactions::{add_transaction_handler, get_account_transactions_handler},
    accounts::{create_account, delete_account, get_all_accounts_handler},
    budgets::{delete_budget, get_budgets_handler},
    categories::{add_category_handler, delete_category, get_categories_handler},
    rules::{delete_rule, get_rules_handler},
    settings::{get_account_settings_handler, get_backup_account_handler, restore_full_backup},
    transactions::delete_transaction,
};

async fn root_redirect() -> Redirect {
    Redirect::to("/accounts")
}

pub fn account_routers() -> Router {
    Router::new()
        .route("/", get(get_all_accounts_handler))
        .route("/", post(create_account))
        .route("/{account_id}", get(get_account_detail))
        .route("/{account_id}", delete(delete_account))
        .route("/{account_id}/rules", get(get_account_rules_handler))
        .route(
            "/{account_id}/transactions",
            get(get_account_transactions_handler),
        )
        .route("/{account_id}/budgets", get(get_account_budgets_handler))
        .route(
            "/{account_id}/rules/preview_apply_rules",
            get(preview_apply_rules),
        )
        .route(
            "/{account_id}/rules/{rule_id}/activate",
            post(activate_rule_handler),
        )
        .route(
            "/{account_id}/rules/{rule_id}/deactivate",
            post(deactivate_rule_handler),
        )
        .route("/{account_id}/transactions", post(add_transaction_handler))
        .route("/{account_id}/budgets", post(add_budget_handler))
        .route("/{account_id}/rules", post(add_account_rule_handler))
        .route("/{account_id}/rules/apply_rules", post(apply_rules))
        .route(
            "/{account_id}/rules/resolve_conflicts",
            post(resolve_conflicts_rules),
        )
}

pub fn category_routers() -> Router {
    Router::new()
        .route("/", get(get_categories_handler))
        .route("/", post(add_category_handler))
        .route("/{category_id}", delete(delete_category))
}

pub fn rule_routers() -> Router {
    Router::new()
        .route("/", get(get_rules_handler))
        .route("/{rule_id}", delete(delete_rule))
}

pub fn budget_routers() -> Router {
    Router::new()
        .route("/", get(get_budgets_handler))
        .route("/{budget_id}", delete(delete_budget))
}

pub fn transaction_routers() -> Router {
    Router::new().route("/{transaction_id}", delete(delete_transaction))
}

pub fn settings_routers() -> Router {
    Router::new()
        .route("/", get(get_account_settings_handler))
        .route("/backup/export", get(get_backup_account_handler))
        .route("/restore", post(restore_full_backup))
}

pub fn router() -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(root_redirect))
        .nest("/accounts", account_routers())
        .nest("/categories", category_routers())
        .nest("/rules", rule_routers())
        .nest("/budgets", budget_routers())
        .nest("/transactions", transaction_routers())
        .nest("/settings", settings_routers())
}
