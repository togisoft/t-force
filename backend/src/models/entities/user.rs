use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    pub name: String,
    pub profile_image: Option<String>,
    pub provider: String,
    pub role: String,
    pub password_hash: Option<String>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// DTOs for user creation and responses
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub name: String,
    pub profile_image: Option<String>,
    pub provider: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponseDto {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub profile_image: Option<String>,
    pub provider: String,
    pub role: String,
    pub is_active: bool,
}

impl From<Model> for UserResponseDto {
    fn from(user: Model) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            profile_image: user.profile_image,
            provider: user.provider,
            role: user.role,
            is_active: user.is_active,
        }
    }
}