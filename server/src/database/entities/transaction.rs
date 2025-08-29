use super::account;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "transactions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub account_id: i32,
    pub value: f64,
    pub description: String,
    pub date: DateTime,
    pub perc_to_exclude: f32,
    pub label: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Account,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Account => Entity::belongs_to(account::Entity)
                .from(Column::AccountId)
                .to(account::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}
