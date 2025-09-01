use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};

use crate::routes::{
    account_rules::{activate_rule_handler, deactivate_rule_handler, get_account_rules_handler},
    accounts::{create_account, get_all_accounts_handler, new_account_form},
    categories::{add_category_handler, get_categories_handler},
    rules::{add_rule_handler, get_rules_handler},
    transactions::{add_transaction_handler, get_account_transactions_handler},
};

async fn root_redirect() -> Redirect {
    Redirect::to("/accounts")
}

// --- Router ---
pub fn router() -> Router {
    Router::new()
        .route("/", get(root_redirect))
        .route("/accounts", get(get_all_accounts_handler))
        .route("/accounts/new", get(new_account_form))
        .route("/accounts", post(create_account))
        .route(
            "/accounts/{account_id}/transactions",
            get(get_account_transactions_handler),
        )
        .route(
            "/accounts/{account_id}/transactions/add",
            post(add_transaction_handler),
        )
        .route("/categories", get(get_categories_handler))
        .route("/categories", post(add_category_handler))
        .route("/rules", get(get_rules_handler))
        .route("/rules", post(add_rule_handler))
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
}
