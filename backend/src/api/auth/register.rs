use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveValue, ActiveModelTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use log::{info, error, debug};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};

use crate::models::{User, UserActiveModel, UserResponseDto, entities::user::Column};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub profile_image: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user: UserResponseDto,
}

#[post("/api/auth/register")]
pub async fn register(
    db: web::Data<DatabaseConnection>,
    user_data: web::Json<RegisterRequest>,
) -> impl Responder {
    let user_data = user_data.into_inner();
    
    debug!("Registering new user with email: {}", user_data.email);
    
    // Check if user with this email already exists
    let existing_user = match User::find()
        .filter(Column::Email.eq(&user_data.email))
        .one(db.get_ref())
        .await {
            Ok(user) => user,
            Err(e) => {
                error!("Database error when checking for existing user: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Database error when checking for existing user"})
                );
            }
        };
    
    if existing_user.is_some() {
        debug!("User with email {} already exists", user_data.email);
        return HttpResponse::BadRequest().json(
            serde_json::json!({"error": "User with this email already exists"})
        );
    }
    
    // Hash the password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(user_data.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            error!("Failed to hash password: {:?}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to hash password"})
            );
        }
    };
    
    // Create new user
    let user_id = Uuid::new_v4();
    let new_user = UserActiveModel {
        id: ActiveValue::Set(user_id),
        email: ActiveValue::Set(user_data.email.clone()),
        name: ActiveValue::Set(user_data.name.clone()),
        profile_image: ActiveValue::Set(user_data.profile_image.clone()),
        provider: ActiveValue::Set("email".to_string()),
        role: ActiveValue::Set("user".to_string()),
        password_hash: ActiveValue::Set(Some(password_hash)),
        password_reset_token: ActiveValue::Set(None),
        password_reset_expires: ActiveValue::Set(None),
        is_active: ActiveValue::Set(true),
        created_at: ActiveValue::Set(chrono::Utc::now()),
        updated_at: ActiveValue::Set(chrono::Utc::now()),
    };
    
    let insert_result = match User::insert(new_user)
        .exec(db.get_ref())
        .await {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to create user: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Failed to create user"})
                );
            }
        };
    
    let user = match User::find_by_id(insert_result.last_insert_id)
        .one(db.get_ref())
        .await {
            Ok(Some(user)) => user,
            Ok(None) => {
                error!("User created but not found when retrieving: {}", user_id);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Failed to retrieve created user"})
                );
            }
            Err(e) => {
                error!("Database error when retrieving created user: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Database error when retrieving created user"})
                );
            }
        };
    
    info!("New user registered successfully: {}", user.id);
    HttpResponse::Created().json(RegisterResponse {
        user: user.into(),
    })
}