# Rust Security Optimizer — Project Memory

## Architecture
- Rust + Axum (port 7000), PostgreSQL via SeaORM, Askama templates
- Three layers: `src/routes/` → `src/database/` → `src/database/entities/`
- Active refactoring: moving SeaORM queries from routes into `src/database/` modules
- Single entry-point router: `src/routes/routes.rs`
- No authentication/authorization on any route (personal/local tool by design)

## Recurring Vulnerability Patterns (verified)

### Panicking `.unwrap()` / `.expect()` in production paths
Extremely widespread. Every handler risks a panic on DB miss or parse failure.
Key locations:
- `database/accounts.rs:11` — `get_account()` `.unwrap()` on `Option`
- `database/settingss.rs:17,28` — double unwrap in settings helpers
- `database/transactions.rs:67-68` — `expect` + `unwrap` in edit path
- `database/rules.rs:91-92` — same pattern in edit path
- `database/categories.rs:59-60` — same pattern in edit path
- `routes/account_detail.rs:93` — `get_account().await.unwrap()` in GET handler
- `routes/account_rules.rs:229` — `Regex::new(...).unwrap()` with user-supplied regex
- `routes/account_settings.rs:189-201` — chained `expect` + `unwrap` in delete handler
- `routes/budgets.rs:95-96` — `expect` + `unwrap` in edit handler
- `routes/uploader.rs:81,86,124,129,154-155,162,168` — multiple panics in file import

### SQL Injection via `Statement::from_string`
- `routes/utilities.rs:45-49` — `reset_sequence()` formats table/sequence names directly
  into a raw SQL string. Called only from `restore_full_backup` with hardcoded values,
  so currently safe, but the function signature accepts arbitrary `&str`.

### ReDoS — Unvalidated User Regex
- `routes/account_rules.rs:229` — user-supplied regex strings from DB compiled without
  timeout/complexity limit. A crafted regex can stall the async executor.

### Out-of-Bounds Panics
- `routes/uploader.rs:81-83,124-126` — `values[date_idx]`, `values[description_idx]`,
  `values[value_idx]` with user-controlled index settings; panics if spreadsheet has
  fewer columns.
- `routes/report.rs:28` — `settings.report_delimiter.as_bytes()[0]` panics if delimiter
  string is empty.
- `routes/account_detail.rs:361,369,379` — slice `[..len-1]` panics if `montly_income`
  is empty (i.e., no transactions in range, or exactly one month).

### Unchecked Multipart / No File Size Limit
- `routes/uploader.rs` and `routes/utilities.rs` — no `ContentLengthLimit` or Axum
  multipart body size limit. Arbitrarily large files can exhaust memory.

### Missing File-Type Validation
- `routes/uploader.rs` — file type determined by client-provided filename extension only;
  no MIME-type check or magic-byte validation.

### `todo!()` in Reachable Code Path
- `routes/uploader.rs:50` — CSV branch calls `todo!()`, which panics at runtime.

### Restore Endpoint — Unauthenticated Full DB Wipe
- `routes/utilities.rs:79` — `restore_full_backup` deletes all data then inserts from
  uploaded JSON. No authentication, no confirmation, no transaction wrapping.
  Any client can wipe and replace the entire database.

### Error Swallowing with `let _ = ...`
- `routes/utilities.rs:191-293` — all restore insert loops use `let _ = X.insert()`,
  silently ignoring insert failures; summary counter still incremented.
- `routes/account_rules.rs:428` — `let _ = the_transaction.update(...)`

### Typo in Route Parameter Name
- `routes/routes.rs:82` — `/{accont_id}/settings/exclude_category` (missing 'u').
  Handler receives `account_id` via `Path<i32>` but the path segment name doesn't match
  any segment, so the path param will always be missing (Axum 0.8 requires name match).

## Performance Patterns

### N+1 Queries in Budget Loop
- `routes/account_detail.rs:106-139` — for each budget: 1 query for categories + 1 query
  for transactions = O(2N) queries where N = budget count.

### Linear Scan for Position Lookup (O(N²) Chart Aggregation)
- `routes/account_detail.rs:280-346` — uses `Vec::contains` + `Vec::position` inside
  a transaction loop. Should use `HashMap`.

### Loading All Transactions into Memory for Backup
- `routes/backup.rs:124` — `transaction::Entity::find().all(db)` loads entire table.
  Fine for small datasets; risk for large ones.

### Empty Vec Slice Subtraction Panic
- `account_detail.rs:361-388` — `montly_income[..montly_income.len() - 1]` panics
  when len == 0 (usize underflow in debug mode, wraps in release to huge slice).

## SeaORM Notes
- All queries use parameterized SeaORM builders — no injection risk from ORM usage.
- `restore_full_backup` deletes tables individually outside a transaction; partial restore
  on failure leaves DB in inconsistent state.
- No `sea_orm` connection pool configuration visible in `main.rs` — uses defaults.

## Dependency Notes (Cargo.toml snapshot, 2026-02-25)
- `axum 0.8.4`, `sea-orm 1.1.16`, `tokio 1`, `regex 1.11.3`, `calamine 0.30.1`
- No `async-std` runtime features in sea-orm config (`runtime-async-std-native-tls`)
  — mismatch with tokio runtime used in `main.rs` (`#[tokio::main]`). This compiles
  because SeaORM will use the tokio executor at runtime, but the feature flag name is
  misleading.

## Linked Detail Files
- See `patterns.md` for extended code snippets and fix patterns.
