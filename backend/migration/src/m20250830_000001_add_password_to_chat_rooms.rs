use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ChatRooms::Table)
                    .add_column(
                        ColumnDef::new(ChatRooms::PasswordHash)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ChatRooms::Table)
                    .drop_column(ChatRooms::PasswordHash)
                    .to_owned(),
            )
            .await
    }
}

/// Reference to the "chat_rooms" table
#[derive(Iden)]
enum ChatRooms {
    Table,
    PasswordHash,
}