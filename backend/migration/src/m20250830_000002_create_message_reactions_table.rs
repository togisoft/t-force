use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MessageReactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MessageReactions::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MessageReactions::MessageId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MessageReactions::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MessageReactions::Emoji)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MessageReactions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_reactions_message_id")
                            .from(MessageReactions::Table, MessageReactions::MessageId)
                            .to(ChatMessages::Table, ChatMessages::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_reactions_user_id")
                            .from(MessageReactions::Table, MessageReactions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // Add a unique constraint to ensure a user can only react once with a specific emoji to a message
                    .index(
                        Index::create()
                            .name("idx_message_reactions_unique")
                            .table(MessageReactions::Table)
                            .col(MessageReactions::MessageId)
                            .col(MessageReactions::UserId)
                            .col(MessageReactions::Emoji)
                            .unique()
                    )
                    .to_owned(),
            )
            .await?;

        // Create an index on message_id for faster reaction retrieval by message
        manager
            .create_index(
                Index::create()
                    .name("idx_message_reactions_message_id")
                    .table(MessageReactions::Table)
                    .col(MessageReactions::MessageId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MessageReactions::Table).to_owned())
            .await
    }
}

/// Reference to the "message_reactions" table
#[derive(Iden)]
enum MessageReactions {
    Table,
    Id,
    MessageId,
    UserId,
    Emoji,
    CreatedAt,
}

/// Reference to the "chat_messages" table for foreign key
#[derive(Iden)]
enum ChatMessages {
    Table,
    Id,
}

/// Reference to the "users" table for foreign key
#[derive(Iden)]
enum Users {
    Table,
    Id,
}