pub use sea_orm_migration::prelude::*;

mod m20250829_131244_create_accounts;
mod m20250829_141149_create_transactions;
mod m20250901_083712_create_categories;
mod m20250901_142239_create_rules;
mod m20250903_143112_create_account_rule;
mod m20250903_152750_create_budget;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250829_131244_create_accounts::Migration),
            Box::new(m20250829_141149_create_transactions::Migration),
            Box::new(m20250901_083712_create_categories::Migration),
            Box::new(m20250901_142239_create_rules::Migration),
            Box::new(m20250903_143112_create_account_rule::Migration),
            Box::new(m20250903_152750_create_budget::Migration),
        ]
    }
}
