use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Budgets::Table)
                    .if_not_exists()
                    .col(pk_auto(Budgets::Id))
                    .col(ColumnDef::new(Budgets::AccountId).integer().not_null())
                    .col(ColumnDef::new(Budgets::Name).string().not_null())
                    .col(ColumnDef::new(Budgets::Value).double().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_budgets_account")
                            .from(Budgets::Table, Budgets::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Budgets::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Budgets {
    Table,
    Id,
    AccountId,
    Name,
    Value,
}

#[derive(Iden)]
enum Accounts {
    Table,
    Id,
}
