use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub ip_address: String,
    pub user_agent: String,
    pub device_type: String,
    pub browser: String,
    pub os: String,
    pub last_active_at: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// DTOs for session operations
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponseDto {
    pub id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub device_type: String,
    pub browser: String,
    pub os: String,
    pub last_active_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
    pub is_current: bool,
}

impl From<Model> for SessionResponseDto {
    fn from(session: Model) -> Self {
        Self {
            id: session.id.to_string(),
            ip_address: session.ip_address,
            user_agent: session.user_agent,
            device_type: session.device_type,
            browser: session.browser,
            os: session.os,
            last_active_at: session.last_active_at,
            created_at: session.created_at,
            is_active: session.is_active,
            is_current: false, // This will be set by the API endpoint
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionDto {
    pub user_id: Uuid,
    pub ip_address: String,
    pub user_agent: String,
    pub device_type: String,
    pub browser: String,
    pub os: String,
}