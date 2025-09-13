use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "message_reactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::chat_message::Entity", from = "Column::MessageId", to = "super::chat_message::Column::Id")]
    ChatMessage,
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::UserId", to = "super::user::Column::Id")]
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

// DTOs for reaction creation and responses
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReactionDto {
    pub message_id: Uuid,
    pub emoji: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionResponseDto {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

// DTO for reaction with user information
#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionWithUserDto {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_profile_image: Option<String>,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

// DTO for reaction counts by emoji
#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionCountDto {
    pub emoji: String,
    pub count: i64,
    pub users: Vec<ReactionUserDto>,
}

// DTO for user who reacted
#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionUserDto {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_profile_image: Option<String>,
}

// DTO for message with reactions
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageWithReactionsDto {
    pub message: super::chat_message::MessageWithUserDto,
    pub reactions: Vec<ReactionCountDto>,
}