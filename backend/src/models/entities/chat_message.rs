use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chat_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::chat_room::Entity", from = "Column::RoomId", to = "super::chat_room::Column::Id")]
    ChatRoom,
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::UserId", to = "super::user::Column::Id")]
    User,
}

impl Related<super::chat_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChatRoom.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// DTOs for message creation and responses
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMessageDto {
    pub room_id: Uuid,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponseDto {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub user_name: Option<String>,
}

// This is a more complete DTO that includes user information
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageWithUserDto {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_profile_image: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}