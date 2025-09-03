use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Rules::Table)
                    .if_not_exists()
                    .col(pk_auto(Rules::Id))
                    .col(ColumnDef::new(Rules::Name).string().not_null())
                    .col(ColumnDef::new(Rules::Label).string().not_null())
                    .col(ColumnDef::new(Rules::Percentage).float().not_null())
                    .col(ColumnDef::new(Rules::CategoryId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rules_category")
                            .from(Rules::Table, Rules::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Rules::Regexpr).string().null())
                    .col(ColumnDef::new(Rules::DateStart).timestamp().null())
                    .col(ColumnDef::new(Rules::DateEnd).timestamp().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Rules::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Categories {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Rules {
    Table,
    Id,
    Name,
    Label,
    Percentage,
    CategoryId,
    Regexpr,
    DateStart,
    DateEnd,
}
