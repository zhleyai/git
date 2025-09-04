use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "commits")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String, // SHA-1 hash
    pub repository_id: Uuid,
    pub parent_ids: Option<String>, // JSON array of parent commit SHAs
    pub tree_id: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: ChronoDateTimeWithTimeZone,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_date: ChronoDateTimeWithTimeZone,
    pub message: String,
    pub content: Vec<u8>, // Raw commit object content
    pub created_at: ChronoDateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::repository::Entity",
        from = "Column::RepositoryId",
        to = "super::repository::Column::Id"
    )]
    Repository,
}

impl Related<super::repository::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Repository.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}