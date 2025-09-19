use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(pk_auto(Settings::Id))
                    .col(ColumnDef::new(Settings::AccountId).integer().not_null())
                    .col(ColumnDef::new(Settings::DateIndex).integer().not_null())
                    .col(
                        ColumnDef::new(Settings::DescriptionIndex)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Settings::ValueIndex).integer().not_null())
                    .col(ColumnDef::new(Settings::StarterString).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_settings_account")
                            .from(Settings::Table, Settings::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Settings {
    Table,
    Id,
    AccountId,
    DateIndex,
    DescriptionIndex,
    ValueIndex,
    StarterString,
}

#[derive(Iden)]
enum Accounts {
    Table,
    Id,
}
