use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::category;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub label: String,
    pub percentage: f32,
    pub category_id: i32,
    pub regexpr: Option<String>,
    pub date_start: Option<NaiveDate>,
    pub date_end: Option<NaiveDate>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::category::Entity",
        from = "Column::CategoryId",
        to = "super::category::Column::Id"
    )]
    Category,
    #[sea_orm(has_many = "super::account_rule::Entity")]
    AccountRule,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        super::account_rule::Relation::Account.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::account_rule::Relation::Rule.def().rev())
    }
}
