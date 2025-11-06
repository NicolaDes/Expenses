use crate::database::account;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub account_id: i32,
    pub date_index: i32,
    pub description_index: i32,
    pub value_index: i32,
    pub starter_string: String,
    pub report_delimiter: String,
    pub report_decimal_separator: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

// Relazione many-to-many con Category attraverso la tabella di join
impl Related<super::category::Entity> for Entity {
    fn to() -> RelationDef {
        super::settings_excluded_category::Relation::Category.def()
    }

    fn via() -> Option<RelationDef> {
        Some(
            super::settings_excluded_category::Relation::Settings
                .def()
                .rev(),
        )
    }
}
