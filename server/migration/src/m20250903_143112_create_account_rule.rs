use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AccountRules::Table)
                    .if_not_exists()
                    .col(pk_auto(AccountRules::Id))
                    .col(ColumnDef::new(AccountRules::AccountId).integer().not_null())
                    .col(ColumnDef::new(AccountRules::RuleId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_account_rule_account")
                            .from(AccountRules::Table, AccountRules::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_account_rule_rule")
                            .from(AccountRules::Table, AccountRules::RuleId)
                            .to(Rules::Table, Rules::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AccountRules::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AccountRules {
    Table,
    Id,
    AccountId,
    RuleId,
}

#[derive(Iden)]
enum Accounts {
    Table,
    Id,
}

#[derive(Iden)]
enum Rules {
    Table,
    Id,
}
