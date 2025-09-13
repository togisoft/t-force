use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use log::{info, error, debug};

use crate::models::{User, UserActiveModel};

#[derive(Debug, Deserialize)]
pub struct SyncUserRequest {
    pub email: String,
    pub name: String,
    pub profile_image: Option<String>,
    pub provider: String,
}

#[derive(Debug, Serialize)]
pub struct SyncUserResponse {
    pub user_id: String,
    pub role: String,
}

#[post("/api/auth/sync")]
pub async fn sync_user(
    db: web::Data<DatabaseConnection>,
    user_data: web::Json<SyncUserRequest>,
) -> impl Responder {
    let user_data = user_data.into_inner();
    
    debug!("Syncing user with email: {}, provider: {}", user_data.email, user_data.provider);
    
    // Check if user exists
    let user = match User::find()
        .filter(crate::models::entities::user::Column::Email.eq(&user_data.email))
        .one(db.get_ref())
        .await {
            Ok(user) => user,
            Err(e) => {
                error!("Database error when finding user by email: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Database error when finding user"})
                );
            }
        };
    
    match user {
        Some(existing_user) => {
            debug!("User found with ID: {}", existing_user.id);
            // User exists, update if needed
            let mut user_active: UserActiveModel = existing_user.clone().into();
            let mut has_changes = false;
            
            // Only update fields if they've changed
            if existing_user.name != user_data.name {
                debug!("Updating user name from '{}' to '{}'", existing_user.name, user_data.name);
                user_active.name = Set(user_data.name);
                has_changes = true;
            }
            
            if existing_user.profile_image != user_data.profile_image {
                debug!("Updating user profile image");
                user_active.profile_image = Set(user_data.profile_image);
                has_changes = true;
            }
            
            // Update the user if there are changes
            if has_changes {
                debug!("User has changes, updating in database");
                let updated_user = match User::update(user_active)
                    .exec(db.get_ref())
                    .await {
                        Ok(user) => user,
                        Err(e) => {
                            error!("Failed to update user: {:?}", e);
                            return HttpResponse::InternalServerError().json(
                                serde_json::json!({"error": "Failed to update user"})
                            );
                        }
                    };
                
                info!("User updated successfully: {}", updated_user.id);
                HttpResponse::Ok().json(SyncUserResponse {
                    user_id: updated_user.id.to_string(),
                    role: updated_user.role,
                })
            } else {
                // No changes needed
                debug!("No changes needed for user: {}", existing_user.id);
                HttpResponse::Ok().json(SyncUserResponse {
                    user_id: existing_user.id.to_string(),
                    role: existing_user.role,
                })
            }
        }
        None => {
            debug!("User not found, creating new user with email: {}", user_data.email);
            // User doesn't exist, create new user
            let user_id = Uuid::new_v4();
            let new_user = UserActiveModel {
                id: Set(user_id),
                email: Set(user_data.email.clone()),
                name: Set(user_data.name.clone()),
                profile_image: Set(user_data.profile_image.clone()),
                provider: Set(user_data.provider.clone()),
                role: Set("user".to_string()),
                password_hash: Set(None),
                password_reset_token: Set(None),
                password_reset_expires: Set(None),
                is_active: Set(true),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
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
            
            info!("New user created successfully: {}", user.id);
            HttpResponse::Created().json(SyncUserResponse {
                user_id: user.id.to_string(),
                role: user.role,
            })
        }
    }
}