
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Add the column without a default value, allowing null temporarily
        manager
            .alter_table(
                Table::alter()
                    .table(ChatRooms::Table)
                    .add_column(
                        ColumnDef::new(ChatRooms::RoomCode)
                            .string()
                            .null() // Allow null initially
                    )
                    .to_owned(),
            )
            .await?;

        // Step 2: Update existing rows with unique values
        // This requires raw SQL since we need to generate unique values for each row
        let update_sql = r#"
            UPDATE chat_rooms
            SET room_code = UPPER(SUBSTRING(gen_random_uuid()::text, 1, 8)) || '_' || UPPER(SUBSTRING(gen_random_uuid()::text, 1, 8))
            WHERE room_code IS NULL
        "#;

        manager.get_connection().execute_unprepared(update_sql).await?;

        // Step 3: Make the column NOT NULL
        manager
            .alter_table(
                Table::alter()
                    .table(ChatRooms::Table)
                    .modify_column(
                        ColumnDef::new(ChatRooms::RoomCode)
                            .string()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await?;

        // Step 4: Add the unique constraint
        manager
            .create_index(
                Index::create()
                    .name("chat_rooms_room_code_key")
                    .table(ChatRooms::Table)
                    .col(ChatRooms::RoomCode)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the index first
        manager
            .drop_index(
                Index::drop()
                    .name("chat_rooms_room_code_key")
                    .to_owned(),
            )
            .await?;

        // Drop the column
        manager
            .alter_table(
                Table::alter()
                    .table(ChatRooms::Table)
                    .drop_column(ChatRooms::RoomCode)
                    .to_owned(),
            )
            .await
    }
}

/// Reference to the "chat_rooms" table
#[derive(Iden)]
enum ChatRooms {
    Table,
    RoomCode,
}