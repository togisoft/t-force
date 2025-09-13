use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RoomMemberships::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RoomMemberships::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(RoomMemberships::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RoomMemberships::RoomId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RoomMemberships::JoinedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_room_memberships_user_id")
                            .from(RoomMemberships::Table, RoomMemberships::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_room_memberships_room_id")
                            .from(RoomMemberships::Table, RoomMemberships::RoomId)
                            .to(ChatRooms::Table, ChatRooms::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_room_memberships_user_room")
                            .col(RoomMemberships::UserId)
                            .col(RoomMemberships::RoomId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RoomMemberships::Table).to_owned())
            .await
    }
}

/// Reference to the "room_memberships" table
#[derive(Iden)]
enum RoomMemberships {
    Table,
    Id,
    UserId,
    RoomId,
    JoinedAt,
}

/// Reference to the "users" table for foreign key
#[derive(Iden)]
enum Users {
    Table,
    Id,
}

/// Reference to the "chat_rooms" table for foreign key
#[derive(Iden)]
enum ChatRooms {
    Table,
    Id,
}