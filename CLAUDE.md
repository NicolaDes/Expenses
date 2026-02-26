# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Expense Manager is a personal finance tracking web application built with **Rust + Axum**, using **PostgreSQL** via **SeaORM**, and **Askama** for server-side HTML templating. The app runs on port 7000 and requires a `DATABASE_URL` environment variable.

## Commands

All Rust commands run from `server/`:

```bash
# Build
cargo build --release

# Run (requires DATABASE_URL env var)
DATABASE_URL=postgres://postgres:postgres@localhost:5432/expenses cargo run

# Check / lint
cargo check
cargo clippy

# Format
cargo fmt
```

### Docker (preferred for development)

```bash
# Build Docker image
./scripts/make-docker-build.sh

# Start / stop full stack (PostgreSQL + backend)
docker compose -f compose.yaml up
docker compose -f compose.yaml down
```

### Database Migrations

Run from `server/migration/`:

```bash
cargo run            # Apply all pending migrations
cargo run -- down    # Rollback last migration
cargo run -- fresh   # Drop all tables and reapply all migrations
cargo run -- status  # Check migration status
cargo run -- generate MIGRATION_NAME  # Create new migration file
```

## Architecture

### Layers

The app follows a clear three-layer structure:

1. **`src/routes/`** — Axum HTTP handlers. Each file corresponds to a resource or view (e.g., `account_detail.rs`, `account_transactions.rs`). Handlers receive the SeaORM `DatabaseConnection` via Axum's `Extension` extractor and render Askama templates or return redirects.

2. **`src/database/`** — Database access functions using SeaORM. The goal (per active refactoring) is that all SeaORM queries live here, not in route handlers. Modules: `accounts`, `transactions`, `categories`, `rules`, `budgets`, `settingss` (note the typo — two s's).

3. **`src/database/entities/`** — SeaORM entity models defining the schema: `Account`, `Transaction`, `Category`, `Rule`, `Budget`, `Settings`, `AccountRule`, `SettingsExcludedCategory`.

### Routing

`src/routes/routes.rs` is the single place where all routes are registered. Routes are grouped into sub-routers (`account_routers()`, `category_routers()`, etc.) and nested under their resource prefix. The root `/` redirects to `/accounts`.

### Data Model Relationships

- `Account` has many `Transaction`, `Budget`, `AccountRule`; one `Settings`
- `Transaction` belongs to one `Category`
- `Rule` is linked to accounts through the `AccountRule` join table
- `Settings` has many `SettingsExcludedCategory` (categories excluded from analytics)

### Frontend

Server-side rendered via Askama templates in `server/templates/`. Static assets (CSS, JS, images) served from `server/static/` via tower-http's `ServeDir`.

## Active Refactoring

The `feature/refactoring` branch has many `// TODO: Move into database modules` comments across `account_transactions.rs`, `account_rules.rs`, `account_detail.rs`, `account_settings.rs`, and `budgets.rs`. The pattern being applied is: extract direct SeaORM query code from route handlers and move it into the corresponding `src/database/*.rs` module as a named function.
