use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::transaction::Entity")]
    Transactions,
    #[sea_orm(has_many = "super::account_rule::Entity")]
    AccountRule,
    #[sea_orm(has_many = "super::budget::Entity")]
    Budgets,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transactions.def()
    }
}

impl Related<super::rule::Entity> for Entity {
    fn to() -> RelationDef {
        super::account_rule::Relation::Rule.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::account_rule::Relation::Account.def().rev())
    }
}

impl Related<super::budget::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Budgets.def()
    }
}
