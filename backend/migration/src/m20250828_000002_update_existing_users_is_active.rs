use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Update all existing users to have is_active = true if it's NULL
        let update_users = Query::update()
            .table(Users::Table)
            .value(Users::IsActive, true)
            .and_where(Expr::col(Users::IsActive).is_null())
            .to_owned();

        manager.exec_stmt(update_users).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // This migration is not reversible in a meaningful way
        // since we can't determine which users had NULL values before
        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Users {
    Table,
    IsActive,
}