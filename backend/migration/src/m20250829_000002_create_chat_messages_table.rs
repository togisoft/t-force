use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatMessages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatMessages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ChatMessages::RoomId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatMessages::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatMessages::Content)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatMessages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ChatMessages::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_messages_room_id")
                            .from(ChatMessages::Table, ChatMessages::RoomId)
                            .to(ChatRooms::Table, ChatRooms::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_messages_user_id")
                            .from(ChatMessages::Table, ChatMessages::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create an index on room_id for faster message retrieval by room
        manager
            .create_index(
                Index::create()
                    .name("idx_chat_messages_room_id")
                    .table(ChatMessages::Table)
                    .col(ChatMessages::RoomId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChatMessages::Table).to_owned())
            .await
    }
}

/// Reference to the "chat_messages" table
#[derive(Iden)]
enum ChatMessages {
    Table,
    Id,
    RoomId,
    UserId,
    Content,
    CreatedAt,
    UpdatedAt,
}

/// Reference to the "chat_rooms" table for foreign key
#[derive(Iden)]
enum ChatRooms {
    Table,
    Id,
}

/// Reference to the "users" table for foreign key
#[derive(Iden)]
enum Users {
    Table,
    Id,
}