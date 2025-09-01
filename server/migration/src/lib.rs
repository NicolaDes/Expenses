pub use sea_orm_migration::prelude::*;

mod m20250829_131244_create_accounts;
mod m20250829_141149_create_transactions;
mod m20250901_083712_create_categories;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250829_131244_create_accounts::Migration),
            Box::new(m20250901_083712_create_categories::Migration),
            Box::new(m20250829_141149_create_transactions::Migration),
        ]
    }
}
