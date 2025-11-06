use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SettingsExcludedCategories::Table)
                    .if_not_exists()
                    .col(pk_auto(SettingsExcludedCategories::Id))
                    .col(
                        ColumnDef::new(SettingsExcludedCategories::SettingsId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SettingsExcludedCategories::CategoryId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_settings_excluded_categories_settings")
                            .from(
                                SettingsExcludedCategories::Table,
                                SettingsExcludedCategories::SettingsId,
                            )
                            .to(Settings::Table, Settings::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_settings_excluded_categories_category")
                            .from(
                                SettingsExcludedCategories::Table,
                                SettingsExcludedCategories::CategoryId,
                            )
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_settings_category_unique")
                            .table(SettingsExcludedCategories::Table)
                            .col(SettingsExcludedCategories::SettingsId)
                            .col(SettingsExcludedCategories::CategoryId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SettingsExcludedCategories::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SettingsExcludedCategories {
    Table,
    Id,
    SettingsId,
    CategoryId,
}

#[derive(DeriveIden)]
enum Settings {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
}
