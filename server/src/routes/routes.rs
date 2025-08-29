use axum::{
    routing::{get, post},
    Router,
};

use crate::routes::{
    accounts::{create_account, get_all_accounts_handler, new_account_form},
    transactions::{add_transaction_handler, get_account_transactions_handler},
};

// --- Router ---
pub fn router() -> Router {
    Router::new()
        .route("/accounts", get(get_all_accounts_handler))
        .route("/accounts/new", get(new_account_form))
        .route("/accounts", post(create_account))
        .route(
            "/accounts/{id}/transactions",
            get(get_account_transactions_handler),
        )
        .route(
            "/accounts/{id}/transactions/add",
            post(add_transaction_handler),
        )
}
