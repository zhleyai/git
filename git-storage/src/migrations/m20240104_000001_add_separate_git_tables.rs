use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create commits table
        manager
            .create_table(
                Table::create()
                    .table(Commits::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Commits::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Commits::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(Commits::ParentIds).string())
                    .col(ColumnDef::new(Commits::TreeId).string().not_null())
                    .col(ColumnDef::new(Commits::AuthorName).string().not_null())
                    .col(ColumnDef::new(Commits::AuthorEmail).string().not_null())
                    .col(ColumnDef::new(Commits::AuthorDate).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Commits::CommitterName).string().not_null())
                    .col(ColumnDef::new(Commits::CommitterEmail).string().not_null())
                    .col(ColumnDef::new(Commits::CommitterDate).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Commits::Message).text().not_null())
                    .col(ColumnDef::new(Commits::Content).binary().not_null())
                    .col(ColumnDef::new(Commits::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-commits-repository")
                            .from(Commits::Table, Commits::RepositoryId)
                            .to(Repository::Table, Repository::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create branches table
        manager
            .create_table(
                Table::create()
                    .table(Branches::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Branches::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Branches::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(Branches::Name).string().not_null())
                    .col(ColumnDef::new(Branches::CommitId).string().not_null())
                    .col(ColumnDef::new(Branches::IsDefault).boolean().not_null().default(false))
                    .col(ColumnDef::new(Branches::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Branches::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-branches-repository")
                            .from(Branches::Table, Branches::RepositoryId)
                            .to(Repository::Table, Repository::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-branches-repo-name")
                            .table(Branches::Table)
                            .col(Branches::RepositoryId)
                            .col(Branches::Name)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create tags table
        manager
            .create_table(
                Table::create()
                    .table(Tags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tags::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Tags::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(Tags::Name).string().not_null())
                    .col(ColumnDef::new(Tags::TargetId).string().not_null())
                    .col(ColumnDef::new(Tags::TargetType).string().not_null())
                    .col(ColumnDef::new(Tags::TagObjectId).string())
                    .col(ColumnDef::new(Tags::TaggerName).string())
                    .col(ColumnDef::new(Tags::TaggerEmail).string())
                    .col(ColumnDef::new(Tags::TaggerDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(Tags::Message).text())
                    .col(ColumnDef::new(Tags::Content).binary())
                    .col(ColumnDef::new(Tags::IsLightweight).boolean().not_null().default(true))
                    .col(ColumnDef::new(Tags::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Tags::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tags-repository")
                            .from(Tags::Table, Tags::RepositoryId)
                            .to(Repository::Table, Repository::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-tags-repo-name")
                            .table(Tags::Table)
                            .col(Tags::RepositoryId)
                            .col(Tags::Name)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create trees table
        manager
            .create_table(
                Table::create()
                    .table(Trees::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Trees::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Trees::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(Trees::Entries).text().not_null())
                    .col(ColumnDef::new(Trees::Size).big_integer().not_null())
                    .col(ColumnDef::new(Trees::Content).binary().not_null())
                    .col(ColumnDef::new(Trees::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-trees-repository")
                            .from(Trees::Table, Trees::RepositoryId)
                            .to(Repository::Table, Repository::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Trees::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Branches::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Commits::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Repository {
    Table,
    Id,
}

#[derive(Iden)]
enum Commits {
    Table,
    Id,
    RepositoryId,
    ParentIds,
    TreeId,
    AuthorName,
    AuthorEmail,
    AuthorDate,
    CommitterName,
    CommitterEmail,
    CommitterDate,
    Message,
    Content,
    CreatedAt,
}

#[derive(Iden)]
enum Branches {
    Table,
    Id,
    RepositoryId,
    Name,
    CommitId,
    IsDefault,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Tags {
    Table,
    Id,
    RepositoryId,
    Name,
    TargetId,
    TargetType,
    TagObjectId,
    TaggerName,
    TaggerEmail,
    TaggerDate,
    Message,
    Content,
    IsLightweight,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Trees {
    Table,
    Id,
    RepositoryId,
    Entries,
    Size,
    Content,
    CreatedAt,
}