use sea_orm::entity::prelude::*;

use crate::database::account;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "budgets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub account_id: i32,
    pub name: String,
    pub value: f64,
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
