use sea_orm::entity::prelude::*;

/// Entity Account senza relazioni
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub balance: f64,
}

/// Enum Relation richiesto dal macro, anche se non ci sono relazioni
#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

/// Implementazione di RelationTrait vuota
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("This entity has no relations");
    }
}

/// Comportamento di default per ActiveModel
impl ActiveModelBehavior for ActiveModel {}
