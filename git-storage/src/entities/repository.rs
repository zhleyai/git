use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "repositories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub created_at: ChronoDateTimeWithTimeZone,
    pub updated_at: ChronoDateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::git_object::Entity")]
    GitObjects,
    #[sea_orm(has_many = "super::git_ref::Entity")]
    GitRefs,
}

impl Related<super::git_object::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GitObjects.def()
    }
}

impl Related<super::git_ref::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GitRefs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}