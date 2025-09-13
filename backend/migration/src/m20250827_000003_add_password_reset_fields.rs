use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add password reset token and expiration fields to users table
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::PasswordResetToken)
                            .string()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Users::PasswordResetExpires)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove password reset fields
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::PasswordResetToken)
                    .drop_column(Users::PasswordResetExpires)
                    .to_owned(),
            )
            .await
    }
}

/// Reference to Users table
#[derive(Iden)]
enum Users {
    Table,
    PasswordResetToken,
    PasswordResetExpires,
}