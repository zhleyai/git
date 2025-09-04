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
                    .table(Commit::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Commit::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Commit::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(Commit::ParentIds).string())
                    .col(ColumnDef::new(Commit::TreeId).string().not_null())
                    .col(ColumnDef::new(Commit::AuthorName).string().not_null())
                    .col(ColumnDef::new(Commit::AuthorEmail).string().not_null())
                    .col(ColumnDef::new(Commit::AuthorDate).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Commit::CommitterName).string().not_null())
                    .col(ColumnDef::new(Commit::CommitterEmail).string().not_null())
                    .col(ColumnDef::new(Commit::CommitterDate).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Commit::Message).text().not_null())
                    .col(ColumnDef::new(Commit::Content).binary().not_null())
                    .col(ColumnDef::new(Commit::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-commit-repository")
                            .from(Commit::Table, Commit::RepositoryId)
                            .to(Repository::Table, Repository::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-commit-repository")
                            .table(Commit::Table)
                            .col(Commit::RepositoryId),
                    )
                    .to_owned(),
            )
            .await?;

        // Create branches table
        manager
            .create_table(
                Table::create()
                    .table(Branch::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Branch::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Branch::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(Branch::Name).string().not_null())
                    .col(ColumnDef::new(Branch::CommitId).string().not_null())
                    .col(ColumnDef::new(Branch::IsDefault).boolean().not_null().default(false))
                    .col(ColumnDef::new(Branch::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Branch::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-branch-repository")
                            .from(Branch::Table, Branch::RepositoryId)
                            .to(Repository::Table, Repository::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-branch-repo-name")
                            .table(Branch::Table)
                            .col(Branch::RepositoryId)
                            .col(Branch::Name)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create tags table
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tag::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Tag::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(Tag::Name).string().not_null())
                    .col(ColumnDef::new(Tag::TargetId).string().not_null())
                    .col(ColumnDef::new(Tag::TargetType).string().not_null())
                    .col(ColumnDef::new(Tag::TagObjectId).string())
                    .col(ColumnDef::new(Tag::TaggerName).string())
                    .col(ColumnDef::new(Tag::TaggerEmail).string())
                    .col(ColumnDef::new(Tag::TaggerDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(Tag::Message).text())
                    .col(ColumnDef::new(Tag::Content).binary())
                    .col(ColumnDef::new(Tag::IsLightweight).boolean().not_null().default(true))
                    .col(ColumnDef::new(Tag::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Tag::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tag-repository")
                            .from(Tag::Table, Tag::RepositoryId)
                            .to(Repository::Table, Repository::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-tag-repo-name")
                            .table(Tag::Table)
                            .col(Tag::RepositoryId)
                            .col(Tag::Name)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create trees table
        manager
            .create_table(
                Table::create()
                    .table(Tree::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tree::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Tree::RepositoryId).uuid().not_null())
                    .col(ColumnDef::new(Tree::Entries).text().not_null())
                    .col(ColumnDef::new(Tree::Size).big_integer().not_null())
                    .col(ColumnDef::new(Tree::Content).binary().not_null())
                    .col(ColumnDef::new(Tree::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tree-repository")
                            .from(Tree::Table, Tree::RepositoryId)
                            .to(Repository::Table, Repository::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx-tree-repository")
                            .table(Tree::Table)
                            .col(Tree::RepositoryId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tree::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Branch::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Commit::Table).to_owned())
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
enum Commit {
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
enum Branch {
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
enum Tag {
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
enum Tree {
    Table,
    Id,
    RepositoryId,
    Entries,
    Size,
    Content,
    CreatedAt,
}