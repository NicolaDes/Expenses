use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // 1. Create labels table
        manager
            .create_table(
                Table::create()
                    .table(Labels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Labels::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Labels::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Insert unique non-empty labels from transactions
        conn.execute_unprepared(
            "INSERT INTO labels (name) \
             SELECT DISTINCT label FROM transactions WHERE label != '' \
             ON CONFLICT (name) DO NOTHING",
        )
        .await?;

        // 3. Insert unique labels from rules not already present
        conn.execute_unprepared(
            "INSERT INTO labels (name) \
             SELECT DISTINCT label FROM rules \
             ON CONFLICT (name) DO NOTHING",
        )
        .await?;

        // 4. Add label_id to transactions (nullable)
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .add_column(ColumnDef::new(Transactions::LabelId).integer().null())
                    .to_owned(),
            )
            .await?;

        // 5. Set FK on transactions from old string
        conn.execute_unprepared(
            "UPDATE transactions \
             SET label_id = (SELECT id FROM labels WHERE name = transactions.label) \
             WHERE label != ''",
        )
        .await?;

        // 6. Add label_id to rules (nullable at first)
        manager
            .alter_table(
                Table::alter()
                    .table(Rules::Table)
                    .add_column(ColumnDef::new(Rules::LabelId).integer().null())
                    .to_owned(),
            )
            .await?;

        // 7. Set FK on rules from old string
        conn.execute_unprepared(
            "UPDATE rules \
             SET label_id = (SELECT id FROM labels WHERE name = rules.label)",
        )
        .await?;

        // 8. Make rules.label_id NOT NULL
        conn.execute_unprepared("ALTER TABLE rules ALTER COLUMN label_id SET NOT NULL")
            .await?;

        // 9. Add FK constraints
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_transactions_label_id")
                    .from(Transactions::Table, Transactions::LabelId)
                    .to(Labels::Table, Labels::Id)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_rules_label_id")
                    .from(Rules::Table, Rules::LabelId)
                    .to(Labels::Table, Labels::Id)
                    .to_owned(),
            )
            .await?;

        // 10. Drop old text columns
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .drop_column(Transactions::Label)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Rules::Table)
                    .drop_column(Rules::Label)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Drop FK constraints
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_transactions_label_id")
                    .table(Transactions::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_rules_label_id")
                    .table(Rules::Table)
                    .to_owned(),
            )
            .await?;

        // Add back string columns
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .add_column(
                        ColumnDef::new(Transactions::Label)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Rules::Table)
                    .add_column(
                        ColumnDef::new(Rules::Label)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        // Restore data
        conn.execute_unprepared(
            "UPDATE transactions \
             SET label = (SELECT name FROM labels WHERE id = transactions.label_id) \
             WHERE label_id IS NOT NULL",
        )
        .await?;
        conn.execute_unprepared(
            "UPDATE rules \
             SET label = (SELECT name FROM labels WHERE id = rules.label_id)",
        )
        .await?;

        // Drop FK columns
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .drop_column(Transactions::LabelId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Rules::Table)
                    .drop_column(Rules::LabelId)
                    .to_owned(),
            )
            .await?;

        // Drop labels table
        manager
            .drop_table(Table::drop().table(Labels::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Labels {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum Transactions {
    Table,
    Label,
    LabelId,
}

#[derive(Iden)]
enum Rules {
    Table,
    Label,
    LabelId,
}
