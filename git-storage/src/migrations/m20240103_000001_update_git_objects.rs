use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add blob_path column for filesystem storage
        // Note: We can't modify existing columns in SQLite, so we'll handle nullable content in code
        manager
            .alter_table(
                Table::alter()
                    .table(GitObject::Table)
                    .add_column(ColumnDef::new(GitObject::BlobPath).string())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop blob_path column
        manager
            .alter_table(
                Table::alter()
                    .table(GitObject::Table)
                    .drop_column(GitObject::BlobPath)
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