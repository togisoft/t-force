use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSessions::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::IpAddress)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::UserAgent)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::DeviceType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::Browser)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::Os)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::LastActiveAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserSessions::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(UserSessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserSessions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_sessions_user")
                            .from(UserSessions::Table, UserSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserSessions::Table).to_owned())
            .await
    }
}

/// User sessions table
#[derive(Iden)]
enum UserSessions {
    Table,
    Id,
    UserId,
    IpAddress,
    UserAgent,
    DeviceType,
    Browser,
    Os,
    LastActiveAt,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

/// Reference to Users table
#[derive(Iden)]
enum Users {
    Table,
    Id,
}