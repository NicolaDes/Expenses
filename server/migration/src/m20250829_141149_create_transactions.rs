use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
enum Accounts {
    // rappresenta la tabella accounts esistente
    Table,
    Id,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transactions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Transactions::AccountId).integer().not_null())
                    .col(ColumnDef::new(Transactions::Value).double().not_null())
                    .col(
                        ColumnDef::new(Transactions::Description)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transactions::Date).timestamp().not_null())
                    .col(
                        ColumnDef::new(Transactions::PercToExclude)
                            .float()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transactions::Label).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transactions_account")
                            .from(Transactions::Table, Transactions::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transactions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Transactions {
    Table,
    Id,
    AccountId,
    Value,
    Description,
    Date,
    PercToExclude,
    Label,
}
