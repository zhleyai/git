use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Modify git_objects table to support blob storage in filesystem
        manager
            .alter_table(
                Table::alter()
                    .table(GitObject::Table)
                    // Make content nullable (blobs will be stored in filesystem)
                    .modify_column(ColumnDef::new(GitObject::Content).binary())
                    // Add blob_path column for filesystem storage
                    .add_column(ColumnDef::new(GitObject::BlobPath).string())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert git_objects table changes
        manager
            .alter_table(
                Table::alter()
                    .table(GitObject::Table)
                    .drop_column(GitObject::BlobPath)
                    .modify_column(ColumnDef::new(GitObject::Content).binary().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum GitObject {
    Table,
    Content,
    BlobPath,
}