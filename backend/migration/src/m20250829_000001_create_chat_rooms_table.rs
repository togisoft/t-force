use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatRooms::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatRooms::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ChatRooms::Name)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatRooms::Description)
                            .string(),
                    )
                    .col(
                        ColumnDef::new(ChatRooms::CreatedBy)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatRooms::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ChatRooms::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_rooms_created_by")
                            .from(ChatRooms::Table, ChatRooms::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChatRooms::Table).to_owned())
            .await
    }
}

/// Reference to the "chat_rooms" table
#[derive(Iden)]
enum ChatRooms {
    Table,
    Id,
    Name,
    Description,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}

/// Reference to the "users" table for foreign key
#[derive(Iden)]
enum Users {
    Table,
    Id,
}