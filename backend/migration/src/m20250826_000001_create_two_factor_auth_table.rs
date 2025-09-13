use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TwoFactorAuth::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TwoFactorAuth::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TwoFactorAuth::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TwoFactorAuth::Secret)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TwoFactorAuth::Enabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(TwoFactorAuth::BackupCodes)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TwoFactorAuth::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TwoFactorAuth::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_two_factor_auth_user")
                            .from(TwoFactorAuth::Table, TwoFactorAuth::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TwoFactorAuth::Table).to_owned())
            .await
    }
}

/// Two-factor authentication table
#[derive(Iden)]
enum TwoFactorAuth {
    Table,
    Id,
    UserId,
    Secret,
    Enabled,
    BackupCodes,
    CreatedAt,
    UpdatedAt,
}

/// Reference to Users table
#[derive(Iden)]
enum Users {
    Table,
    Id,
}