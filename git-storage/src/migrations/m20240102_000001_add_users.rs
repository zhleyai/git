use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(User::Username).string().not_null().unique_key())
                    .col(ColumnDef::new(User::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(User::PasswordHash).string().not_null())
                    .col(ColumnDef::new(User::FullName).string())
                    .col(ColumnDef::new(User::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(User::IsAdmin).boolean().not_null().default(false))
                    .col(ColumnDef::new(User::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(User::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // Add owner_id column to repositories table  
        manager
            .alter_table(
                Table::alter()
                    .table(Repository::Table)
                    .add_column(ColumnDef::new(Repository::OwnerId).uuid())
                    .to_owned(),
            )
            .await?;

        // Add is_private column to repositories table
        manager
            .alter_table(
                Table::alter()
                    .table(Repository::Table)
                    .add_column(ColumnDef::new(Repository::IsPrivate).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove columns from repositories table
        manager
            .alter_table(
                Table::alter()
                    .table(Repository::Table)
                    .drop_column(Repository::IsPrivate)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Repository::Table)
                    .drop_column(Repository::OwnerId)
                    .to_owned(),
            )
            .await?;

        // Drop users table
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum User {
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
    FullName,
    IsActive,
    IsAdmin,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Repository {
    Table,
    OwnerId,
    IsPrivate,
}