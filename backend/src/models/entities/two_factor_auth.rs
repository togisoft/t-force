use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "two_factor_auth")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub secret: String,
    pub enabled: bool,
    #[sea_orm(column_type = "Json", nullable)]
    pub backup_codes: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// DTOs for 2FA operations
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorSetupDto {
    pub secret: String,
    pub qr_code_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorVerifyDto {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorStatusDto {
    pub enabled: bool,
    pub backup_codes_remaining: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorBackupCodesDto {
    pub backup_codes: Vec<String>,
}

impl From<Model> for TwoFactorStatusDto {
    fn from(two_factor: Model) -> Self {
        Self {
            enabled: two_factor.enabled,
            backup_codes_remaining: two_factor.backup_codes
                .as_ref() // Option<Value> -> Option<&Value>
                .and_then(|codes_value| codes_value.as_array()) // Option<&Value> -> Option<&Vec<Value>>
                .map(|array| array.len()),
        }
    }
}