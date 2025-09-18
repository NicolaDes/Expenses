use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

use crate::routes::{
    account_budgets::{add_budget_handler, get_account_budgets_handler},
    account_detail::get_account_detail,
    account_index::{create_account, get_all_accounts_handler},
    account_rules::{
        activate_rule_handler, add_account_rule_handler, apply_rules, deactivate_rule_handler,
        get_account_rules_handler, preview_apply_rules, resolve_conflicts_rules,
    },
    account_transactions::{add_transaction_handler, get_account_transactions_handler},
    categories::{add_category_handler, get_categories_handler},
    index::get_index,
};

async fn root_redirect() -> Redirect {
    Redirect::to("/accounts")
}

pub fn router() -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(root_redirect))
        .route("/accounts", get(get_all_accounts_handler))
        .route("/accounts", post(create_account))
        .route("/accounts/{account_id}", get(get_account_detail))
        .route(
            "/accounts/{account_id}/rules",
            get(get_account_rules_handler),
        )
        .route(
            "/accounts/{account_id}/rules/{rule_id}/activate",
            post(activate_rule_handler),
        )
        .route(
            "/accounts/{account_id}/rules/{rule_id}/deactivate",
            post(deactivate_rule_handler),
        )
        .route(
            "/accounts/{account_id}/transactions",
            get(get_account_transactions_handler),
        )
        .route(
            "/accounts/{account_id}/transactions",
            post(add_transaction_handler),
        )
        .route(
            "/accounts/{account_id}/budgets",
            get(get_account_budgets_handler),
        )
        .route("/accounts/{account_id}/budgets", post(add_budget_handler))
        .route(
            "/accounts/{account_id}/rules",
            post(add_account_rule_handler),
        )
        .route("/categories", get(get_categories_handler))
        .route("/categories", post(add_category_handler))
        .route("/rules", get(get_index))
        .route("/budgets", get(get_index))
        .route("/accounts/{account_id}/settings", get(get_index))
        .route(
            "/accounts/{account_id}/rules/preview_apply_rules",
            get(preview_apply_rules),
        )
        .route(
            "/accounts/{account_id}/rules/apply_rules",
            post(apply_rules),
        )
        .route(
            "/accounts/{account_id}/rules/resolve_conflicts",
            post(resolve_conflicts_rules),
        )
}
