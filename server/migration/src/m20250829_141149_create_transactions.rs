use sea_orm_migration::prelude::*;

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
                    .col(ColumnDef::new(Transactions::CategoryId).integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transactions_category")
                            .from(Transactions::Table, Transactions::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
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
enum Accounts {
    Table,
    Id,
}

#[derive(Iden)]
enum Categories {
    Table,
    Id,
}

#[derive(Iden)]
enum Transactions {
    Table,
    Id,
    AccountId,
    CategoryId,
    Value,
    Description,
    Date,
    PercToExclude,
    Label,
}
