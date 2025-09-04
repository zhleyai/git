use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create repositories table
        manager
            .create_table(
                Table::create()
                    .table(Repository::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Repository::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Repository::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Repository::Description).text())
                    .col(ColumnDef::new(Repository::DefaultBranch).string().not_null())
                    .col(ColumnDef::new(Repository::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Repository::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // Create git_objects table
        manager
            .create_table(
                Table::create()
                    .table(GitObject::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GitObject::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(GitObject::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(GitObject::ObjectType).string().not_null())
                    .col(ColumnDef::new(GitObject::Size).big_integer().not_null())
                    .col(ColumnDef::new(GitObject::Content).binary().not_null())
                    .col(ColumnDef::new(GitObject::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-gitobject-repository")
                            .from(GitObject::Table, GitObject::RepositoryId)
                            .to(Repository::Table, Repository::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create git_refs table
        manager
            .create_table(
                Table::create()
                    .table(GitRef::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GitRef::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(GitRef::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(GitRef::Name).string().not_null())
                    .col(ColumnDef::new(GitRef::Target).string().not_null())
                    .col(ColumnDef::new(GitRef::IsSymbolic).boolean().not_null())
                    .col(ColumnDef::new(GitRef::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(GitRef::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-gitref-repository")
                            .from(GitRef::Table, GitRef::RepositoryId)
                            .to(Repository::Table, Repository::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-gitref-repo-name")
                            .table(GitRef::Table)
                            .col(GitRef::RepositoryId)
                            .col(GitRef::Name)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GitRef::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GitObject::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Repository::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Repository {
    Table,
    Id,
    Name,
    Description,
    DefaultBranch,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum GitObject {
    Table,
    Id,
    RepositoryId,
    ObjectType,
    Size,
    Content,
    CreatedAt,
}

#[derive(Iden)]
enum GitRef {
    Table,
    Id,
    RepositoryId,
    Name,
    Target,
    IsSymbolic,
    CreatedAt,
    UpdatedAt,
}