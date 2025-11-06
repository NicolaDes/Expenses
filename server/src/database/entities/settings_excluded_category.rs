use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "settings_excluded_categories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub settings_id: i32,
    pub category_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::settings::Entity",
        from = "Column::SettingsId",
        to = "super::settings::Column::Id"
    )]
    Settings,

    #[sea_orm(
        belongs_to = "super::category::Entity",
        from = "Column::CategoryId",
        to = "super::category::Column::Id"
    )]
    Category,
}

impl ActiveModelBehavior for ActiveModel {}
