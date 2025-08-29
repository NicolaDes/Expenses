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
pub enum Relation {
    Transactions,
}

/// Implementazione di RelationTrait vuota
impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Transactions => Entity::has_many(super::transaction::Entity).into(),
        }
    }
}

/// Comportamento di default per ActiveModel
impl ActiveModelBehavior for ActiveModel {}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transactions.def()
    }
}
