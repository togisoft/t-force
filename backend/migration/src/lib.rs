pub use sea_orm_migration::prelude::*;

mod m20250824_000001_create_users_table;
mod m20250826_000001_create_two_factor_auth_table;
mod m20250827_000001_create_user_sessions_table;
mod m20250827_000002_add_default_admin_user;
mod m20250827_000003_add_password_reset_fields;
mod m20250828_000001_add_is_active_to_users;
mod m20250828_000002_update_existing_users_is_active;
mod m20250829_000001_create_chat_rooms_table;
mod m20250829_000002_create_chat_messages_table;
mod m20250830_000001_add_password_to_chat_rooms;
mod m20250830_000002_create_message_reactions_table;
mod m20250830_000003_add_room_code_to_chat_rooms;
mod m20250830_000004_create_room_memberships_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250824_000001_create_users_table::Migration),
            Box::new(m20250826_000001_create_two_factor_auth_table::Migration),
            Box::new(m20250827_000001_create_user_sessions_table::Migration),
            Box::new(m20250827_000003_add_password_reset_fields::Migration),
            Box::new(m20250827_000002_add_default_admin_user::Migration),
            Box::new(m20250828_000001_add_is_active_to_users::Migration),
            Box::new(m20250828_000002_update_existing_users_is_active::Migration),
            Box::new(m20250829_000001_create_chat_rooms_table::Migration),
            Box::new(m20250829_000002_create_chat_messages_table::Migration),
            Box::new(m20250830_000001_add_password_to_chat_rooms::Migration),
            Box::new(m20250830_000002_create_message_reactions_table::Migration),
            Box::new(m20250830_000003_add_room_code_to_chat_rooms::Migration),
            Box::new(m20250830_000004_create_room_memberships_table::Migration),
        ]
    }
}