use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "room_memberships")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::UserId", to = "super::user::Column::Id")]
    User,
    #[sea_orm(belongs_to = "super::chat_room::Entity", from = "Column::RoomId", to = "super::chat_room::Column::Id")]
    ChatRoom,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::chat_room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChatRoom.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// DTOs for room membership
#[derive(Debug, Serialize, Deserialize)]
pub struct RoomMembershipResponseDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

impl From<Model> for RoomMembershipResponseDto {
    fn from(membership: Model) -> Self {
        Self {
            id: membership.id,
            user_id: membership.user_id,
            room_id: membership.room_id,
            joined_at: membership.joined_at,
        }
    }
}