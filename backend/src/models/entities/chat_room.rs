use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chat_rooms")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub password_hash: Option<String>,
    pub room_code: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::chat_message::Entity")]
    ChatMessage,
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::CreatedBy", to = "super::user::Column::Id")]
    User,
}

impl Related<super::chat_message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChatMessage.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// DTOs for room creation and responses
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomDto {
    pub name: String,
    pub description: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomResponseDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub is_protected: bool,
    pub is_owner: bool,
    pub room_code: String,
    pub user_count: u64,
}

impl From<Model> for RoomResponseDto {
    fn from(room: Model) -> Self {
        Self {
            id: room.id,
            name: room.name,
            description: room.description,
            created_by: room.created_by,
            created_at: room.created_at,
            is_protected: room.password_hash.is_some(),
            is_owner: false, // Will be set separately based on user context
            room_code: room.room_code,
            user_count: 0, // Will be set separately
        }
    }
}