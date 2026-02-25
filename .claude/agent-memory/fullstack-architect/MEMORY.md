# Fullstack Architect Agent Memory

## Codebase Quick Reference
- Server root: `/home/sphero/code/expenses/server/`
- Routes: `src/routes/` (12 handler files + mod + routes + common + backup)
- Database: `src/database/` (6 modules: accounts, transactions, categories, rules, budgets, settingss)
- Entities: `src/database/entities/` (8 entities: account, transaction, category, rule, budget, settings, account_rule, settings_excluded_category)
- Templates: `server/templates/` (13 files; base.html + base_account.html are inherited by account sub-pages)

## Key Architectural Findings (Feb 2026 audit)
See `patterns.md` for full detail. Summary:
- 46 TODOs "Move into database modules" across 5 route files
- Route files with ZERO violations (already clean): accounts.rs, categories.rs, rules.rs, transactions.rs, account_budgets.rs
- Worst violators: account_rules.rs (18 TODOs), account_settings.rs (10), account_detail.rs (8), account_transactions.rs (5), budgets.rs (4)
- `backup.rs` and `report.rs` in routes/ contain SeaORM queries with NO TODO markers — unlabeled violations

## Known Bugs / Typos to Fix
- `routes.rs` line 82: route parameter `{accont_id}` (missing 'u') — `add_excluded_category_handler` is dead/unreachable
- `backup.rs` line 20: field name `exlcuded_categories` (transposed 'l' and 'c') — appears in DTO struct and serialized JSON, breaking backup format
- `account_detail.rs`: pervasive `montly` typo (missing 'h') in ChartData struct fields and variable names — exposed in JSON API

## Error Handling Anti-patterns Observed
- `html.render().unwrap()` in 9 route handlers — Askama render failure panics the handler
- `settingss::get_settings_for_account` uses `.unwrap()` not `?` — propagates panic instead of Result
- `accounts::get_account` uses `.one(db).await?.unwrap()` — unwrap on None instead of returning proper error
- Extensive use of `.expect()` in route handlers and database modules

## Named Convention Issues
- Database module: `settingss` (double-s) — intentional per CLAUDE.md but confusing; confirm before renaming
- `backup.rs` lives in `src/routes/` but contains no route handlers — it is pure data/DTO code; belongs elsewhere

## Commented-Out Dead Code
- `transactions.rs`: `create_transaction`, `get_transactions` commented out
- `rules.rs`: `create_rule` commented out
- `budgets.rs`: `delete_budget`, `edit_budget`, `get_budgets` commented out
- `settingss.rs`: `create_settings`, `delete_settings`, `edit_settings` commented out
- `categories.rs`: `get_category` commented out

## Template Patterns
- `base.html` is the root; `base_account.html` extends it and adds account sub-navbar
- All account sub-pages extend `base_account.html` and receive `account`, `menu`, `sub_menu`
- `html.render().unwrap()` is the universal rendering call — none use `?` propagation

## Unlabeled Violations (no TODO comment)
- `report.rs`: direct SeaORM queries in `get_splittable_expenses_report`
- `backup.rs`: all of `get_full_backup` and `restore_full_backup` contain direct entity queries
- `uploader.rs`: `settings::Entity::find()` query + `transaction::ActiveModel` insert loop
- `utilities.rs`: `restore_full_backup` uses entity inserts directly (inherits from backup.rs pattern)
