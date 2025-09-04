use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String, // Tag name (e.g., "v1.0.0")
    pub target_id: String, // SHA-1 of the object this tag points to (usually a commit)
    pub target_type: String, // Type of object: "commit", "tree", "blob", or "tag"
    pub tag_object_id: Option<String>, // SHA-1 of the tag object itself (for annotated tags)
    pub tagger_name: Option<String>, // For annotated tags
    pub tagger_email: Option<String>, // For annotated tags
    pub tagger_date: Option<ChronoDateTimeWithTimeZone>, // For annotated tags
    pub message: Option<String>, // For annotated tags
    pub content: Option<Vec<u8>>, // Raw tag object content (for annotated tags)
    pub is_lightweight: bool, // True for lightweight tags, false for annotated tags
    pub created_at: ChronoDateTimeWithTimeZone,
    pub updated_at: ChronoDateTimeWithTimeZone,
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