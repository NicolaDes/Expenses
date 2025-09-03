use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "account_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub account_id: i32,
    pub rule_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,
    #[sea_orm(
        belongs_to = "super::rule::Entity",
        from = "Column::RuleId",
        to = "super::rule::Column::Id"
    )]
    Rule,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::rule::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rule.def()
    }
}
