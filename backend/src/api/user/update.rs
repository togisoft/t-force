use actix_web::{put, web, HttpResponse, Responder, HttpRequest, http::header};
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};
use log::{debug, error, warn};
use jsonwebtoken::{decode, DecodingKey, Validation, errors::ErrorKind};
use uuid::Uuid;
use argon2::{password_hash::{
    rand_core::OsRng,
    PasswordHasher, SaltString
}, Argon2, PasswordVerifier};

use crate::auth::Claims;
use crate::api::auth::is_token_blacklisted;
use crate::models::entities::{User, UserActiveModel, user::Column};

#[derive(Deserialize)]
pub struct UpdateUsernameRequest {
    name: String,
}

#[derive(Deserialize)]
pub struct UpdatePasswordRequest {
    current_password: String,
    new_password: String,
}

#[derive(Serialize)]
pub struct UpdateResponse {
    success: bool,
    message: String,
}

// Extract user ID from token
async fn extract_user_id(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    // Try to extract the token from the secure `HttpOnly` cookie first.
    let mut token_str: Option<String> = None;
    
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found 'auth_token' cookie");
        token_str = Some(cookie.value().to_string());
    }

    // If no cookie is found, fall back to checking the Authorization header.
    if token_str.is_none() {
        debug!("No 'auth_token' cookie, checking Authorization header");
        if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    token_str = Some(auth_str[7..].to_string());
                }
            }
        }
    }

    // If no token was found in either place, return an Unauthorized error.
    let token = match token_str {
        Some(t) => t,
        None => {
            warn!("Authentication token not found");
            return Err(HttpResponse::Unauthorized().json(serde_json::json!({
                "success": false,
                "message": "Authentication token is missing"
            })));
        }
    };

    if is_token_blacklisted(&token) {
        warn!("Blacklisted token used");
        return Err(HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "message": "Token has been invalidated"
        })));
    }

    let jwt_secret = std::env::var("NEXTAUTH_SECRET").expect("NEXTAUTH_SECRET must be set");
    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(e) => match *e.kind() {
            ErrorKind::ExpiredSignature => {
                return Err(HttpResponse::Unauthorized().json(serde_json::json!({
                    "success": false,
                    "message": "Token has expired"
                })));
            }
            _ => {
                warn!("Invalid token: {:?}", e);
                return Err(HttpResponse::Unauthorized().json(serde_json::json!({
                    "success": false,
                    "message": "Invalid token"
                })));
            }
        },
    };

    let claims = token_data.claims;
    let user_id_str = match claims.backend_user_id {
        Some(id) => id,
        None => return Err(HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "message": "Token missing user ID"
        }))),
    };

    match Uuid::parse_str(&user_id_str) {
        Ok(id) => Ok(id),
        Err(_) => Err(HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "message": "Invalid user ID format"
        }))),
    }
}

#[put("/api/user/update-username")]
pub async fn update_username(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    update_req: web::Json<UpdateUsernameRequest>,
) -> impl Responder {
    // Validate username
    if update_req.name.trim().is_empty() {
        return HttpResponse::BadRequest().json(UpdateResponse {
            success: false,
            message: "Username cannot be empty".to_string(),
        });
    }

    // Extract user ID from token
    let user_id = match extract_user_id(&req).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    // Find the user
    match User::find_by_id(user_id).one(db.as_ref()).await {
        Ok(Some(user)) => {
            // Update the username
            let mut user_active: UserActiveModel = user.into();
            user_active.name = Set(update_req.name.clone());

            match user_active.update(db.as_ref()).await {
                Ok(_) => {
                    HttpResponse::Ok().json(UpdateResponse {
                        success: true,
                        message: "Username updated successfully".to_string(),
                    })
                }
                Err(e) => {
                    error!("Database error updating username: {:?}", e);
                    HttpResponse::InternalServerError().json(UpdateResponse {
                        success: false,
                        message: "Failed to update username".to_string(),
                    })
                }
            }
        }
        Ok(None) => {
            error!("User with ID {} not found in DB", user_id);
            HttpResponse::NotFound().json(UpdateResponse {
                success: false,
                message: "User not found".to_string(),
            })
        }
        Err(e) => {
            error!("Database error fetching user {}: {:?}", user_id, e);
            HttpResponse::InternalServerError().json(UpdateResponse {
                success: false,
                message: "Database error".to_string(),
            })
        }
    }
}

#[put("/api/user/update-password")]
pub async fn update_password(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    update_req: web::Json<UpdatePasswordRequest>,
) -> impl Responder {
    // Validate password
    if update_req.new_password.len() < 8 {
        return HttpResponse::BadRequest().json(UpdateResponse {
            success: false,
            message: "Password must be at least 8 characters long".to_string(),
        });
    }

    // Extract user ID from token
    let user_id = match extract_user_id(&req).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    // Find the user
    match User::find_by_id(user_id).one(db.as_ref()).await {
        Ok(Some(user)) => {
            // Verify current password
            let argon2 = Argon2::default();
            let password_hash = match user.password_hash {
                Some(ref hash) => hash,
                None => {
                    return HttpResponse::BadRequest().json(UpdateResponse {
                        success: false,
                        message: "Cannot update password for OAuth users".to_string(),
                    });
                }
            };

            let is_valid = argon2::password_hash::PasswordHash::new(password_hash)
                .and_then(|parsed_hash| {
                    argon2.verify_password(update_req.current_password.as_bytes(), &parsed_hash)
                })
                .is_ok();

            if !is_valid {
                return HttpResponse::BadRequest().json(UpdateResponse {
                    success: false,
                    message: "Current password is incorrect".to_string(),
                });
            }

            // Hash the new password
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = match argon2.hash_password(update_req.new_password.as_bytes(), &salt) {
                Ok(hash) => hash.to_string(),
                Err(e) => {
                    error!("Error hashing password: {:?}", e);
                    return HttpResponse::InternalServerError().json(UpdateResponse {
                        success: false,
                        message: "Error processing password".to_string(),
                    });
                }
            };

            // Update the password
            let mut user_active: UserActiveModel = user.into();
            user_active.password_hash = Set(Some(password_hash));

            match user_active.update(db.as_ref()).await {
                Ok(_) => {
                    HttpResponse::Ok().json(UpdateResponse {
                        success: true,
                        message: "Password updated successfully".to_string(),
                    })
                }
                Err(e) => {
                    error!("Database error updating password: {:?}", e);
                    HttpResponse::InternalServerError().json(UpdateResponse {
                        success: false,
                        message: "Failed to update password".to_string(),
                    })
                }
            }
        }
        Ok(None) => {
            error!("User with ID {} not found in DB", user_id);
            HttpResponse::NotFound().json(UpdateResponse {
                success: false,
                message: "User not found".to_string(),
            })
        }
        Err(e) => {
            error!("Database error fetching user {}: {:?}", user_id, e);
            HttpResponse::InternalServerError().json(UpdateResponse {
                success: false,
                message: "Database error".to_string(),
            })
        }
    }
}
