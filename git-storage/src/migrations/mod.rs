pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_tables;
mod m20240102_000001_add_users;
mod m20240103_000001_update_git_objects;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_tables::Migration),
            Box::new(m20240102_000001_add_users::Migration),
            Box::new(m20240103_000001_update_git_objects::Migration),
        ]
    }
}