# Codebase Patterns Reference

## Database Module Style (what "done right" looks like)
- Return `anyhow::Result<T>` — never panic, never StatusCode
- Use `context("...")` from `anyhow::Context` on `.await?` chains
- Function signatures: `pub async fn verb_noun(db: &DatabaseConnection, ...) -> anyhow::Result<T>`
- Example of correct style: `src/database/accounts.rs`, `src/database/categories.rs`

## Route Handler Style (what "done right" looks like)
- Extract path/form/query with Axum extractors only
- Call database module functions
- Map errors to StatusCode with `.map_err(|e| { eprintln!(...); StatusCode::X })?`
- Render template and return Html
- Example of correct style: `src/routes/accounts.rs`, `src/routes/transactions.rs`

## SeaORM Query Grouping Rules
- `get_X` / `get_Xs_for_account` → `src/database/Xs.rs`
- `create_X` → `src/database/Xs.rs`
- `delete_X` → `src/database/Xs.rs`
- `edit_X` / `update_X` → `src/database/Xs.rs`
- Cross-entity joins (e.g., transactions with categories) → belong in `src/database/transactions.rs` as named functions

## N+1 Pattern in account_detail.rs
Lines 106-139: Loops over budgets, runs two queries per budget iteration (find categories, find transactions).
Should be refactored to a batch query approach in `src/database/budgets.rs` or `src/database/transactions.rs`.

## apply_rules Pattern (account_rules.rs)
Both `preview_apply_rules` and `apply_rules` (lines 255-382) repeat the same two DB queries:
1. uncategorized transactions for account
2. active rules for account
Should be extracted into `src/database/rules.rs::get_active_rules_for_account` and `src/database/transactions.rs::get_uncategorized_transactions_for_account`.

## Backup Architecture Issue
`src/routes/backup.rs` — despite living in routes/, it has no HTTP handler. It defines DTOs and `get_full_backup()` which is a pure data-access function that should live in `src/database/backup.rs` or similar.

## Missing Database Module: account_rules
There is no `src/database/account_rules.rs`. The `account_rule` entity operations (activate, deactivate, get active rules for account) are scattered across route handlers.
Needed functions:
- `activate_rule_for_account(db, account_id, rule_id) -> Result<()>`
- `deactivate_rule_for_account(db, account_id, rule_id) -> Result<()>`
- `get_active_rules_for_account(db, account_id) -> Result<Vec<rule::Model>>`
- `get_active_rule_ids_for_account(db, account_id) -> Result<HashSet<i32>>`
