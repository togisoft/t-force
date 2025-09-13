use actix_web::{web, HttpResponse, post, HttpRequest, Error};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveValue, QueryFilter, ColumnTrait, PaginatorTrait};
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::auth::{AuthUser, JwtAuth};
use crate::models::entities::{ChatRoom, RoomResponseDto, RoomMembership, RoomMembershipActiveModel, User};

use argon2::{
    password_hash::{
        PasswordHash,
        PasswordVerifier
    },
    Argon2
};

#[derive(Deserialize)]
pub struct JoinRoomByCodeRequest {
    pub room_code: String,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct JoinRoomByCodeResponse {
    pub success: bool,
    pub message: String,
    pub room: Option<RoomResponseDto>,
}

#[post("/api/chat/rooms/join-by-code")]
pub async fn join_room_by_code(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    join_data: web::Json<JoinRoomByCodeRequest>,
) -> Result<HttpResponse, Error> {
    // Extract token strictly from cookie (cookie-based auth)
    let token = match req.cookie("auth_token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            return Ok(HttpResponse::Unauthorized().json(JoinRoomByCodeResponse {
                success: false,
                message: "Unauthorized: Missing auth_token cookie".to_string(),
                room: None,
            }));
        }
    };

    // Validate token and get user
    let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
        Ok(claims) => claims,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(JoinRoomByCodeResponse {
                success: false,
                message: "Unauthorized: Invalid authentication token".to_string(),
                room: None,
            }));
        }
    };

    // Ensure backend_user_id is present and valid
    let backend_user_id = match claims.backend_user_id {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(JoinRoomByCodeResponse {
                success: false,
                message: "Unauthorized: Missing user ID in token".to_string(),
                room: None,
            }));
        }
    };

    let user_id = match Uuid::parse_str(&backend_user_id) {
        Ok(id) => id,
        Err(e) => {
            log::error!(
                "Failed to parse backend_user_id into UUID. Value: '{}', Error: {}",
                backend_user_id,
                e
            );
            return Ok(HttpResponse::Unauthorized().json(JoinRoomByCodeResponse {
                success: false,
                message: "Unauthorized: Invalid user ID format in token".to_string(),
                room: None,
            }));
        }
    };

    // Create AuthUser from claims
    let auth = AuthUser {
        id: user_id,
        email: claims.email,
        name: claims.name,
        role: claims.user_role.unwrap_or_else(|| "user".to_string()),
        profile_image: claims.profile_image,
    };

    // Ensure user exists to satisfy FK constraint
    match User::find_by_id(auth.id).one(db.get_ref()).await {
        Ok(Some(_)) => { /* exists */ },
        Ok(None) => {
            log::error!("join_room_by_code: Authenticated user not found in DB: {}", auth.id);
            return Ok(HttpResponse::Unauthorized().json(JoinRoomByCodeResponse {
                success: false,
                message: "Unauthorized: user does not exist".to_string(),
                room: None,
            }));
        },
        Err(e) => {
            log::error!("join_room_by_code: Error querying user {}: {}", auth.id, e);
            return Ok(HttpResponse::InternalServerError().json(JoinRoomByCodeResponse {
                success: false,
                message: "Server error while validating user".to_string(),
                room: None,
            }));
        }
    }

    let room_code = join_data.room_code.trim().to_uppercase();

    match ChatRoom::find()
        .filter(crate::models::entities::chat_room::Column::RoomCode.eq(room_code))
        .one(db.get_ref())
        .await {
        Ok(Some(room)) => {
            // --- MODIFICATION START: Argon2 Password Verification ---
            if let Some(password_hash) = &room.password_hash {
                let provided_password = match &join_data.password {
                    Some(p) => p,
                    None => {
                        return Ok(HttpResponse::BadRequest().json(JoinRoomByCodeResponse {
                            success: false,
                            message: "This room requires a password.".to_string(),
                            room: None,
                        }));
                    }
                };

                // Parse the hash from the database.
                let parsed_hash = match PasswordHash::new(password_hash) {
                    Ok(hash) => hash,
                    Err(e) => {
                        log::error!("Failed to parse password hash from DB for room {}: {}", room.id, e);
                        // This indicates a problem with the stored hash, which is a server error.
                        return Ok(HttpResponse::InternalServerError().json(JoinRoomByCodeResponse {
                            success: false,
                            message: "Server configuration error.".to_string(),
                            room: None,
                        }));
                    }
                };

                // Verify the password.
                match Argon2::default().verify_password(provided_password.as_bytes(), &parsed_hash) {
                    Ok(_) => {
                        // Password is correct, proceed.
                        log::info!("Password correct for room ID: {}", room.id);
                    },
                    Err(argon2::password_hash::Error::Password) => {
                        // This specific error means the password did not match.
                        return Ok(HttpResponse::Unauthorized().json(JoinRoomByCodeResponse {
                            success: false,
                            message: "Invalid password.".to_string(),
                            room: None,
                        }));
                    },
                    Err(e) => {
                        // Any other error is an internal server issue.
                        log::error!("Argon2 verification failed for room {}: {}", room.id, e);
                        return Ok(HttpResponse::InternalServerError().json(JoinRoomByCodeResponse {
                            success: false,
                            message: "Error during password verification.".to_string(),
                            room: None,
                        }));
                    }
                }
            }

            let existing_membership = RoomMembership::find()
                .filter(crate::models::entities::room_membership::Column::UserId.eq(auth.id))
                .filter(crate::models::entities::room_membership::Column::RoomId.eq(room.id))
                .one(db.get_ref())
                .await;

            match existing_membership {
                Ok(Some(_)) => {
                    // Get user count for this room
                    let user_count_result = RoomMembership::find()
                        .filter(crate::models::entities::room_membership::Column::RoomId.eq(room.id))
                        .count(db.get_ref())
                        .await;
                    
                    let user_count = match user_count_result {
                        Ok(count) => count,
                        Err(e) => {
                            log::error!("Failed to get user count for room {}: {}", room.id, e);
                            0
                        }
                    };
                    
                    let room_response = RoomResponseDto {
                        id: room.id,
                        name: room.name,
                        description: room.description,
                        created_by: room.created_by,
                        created_at: room.created_at,
                        is_protected: room.password_hash.is_some(),
                        is_owner: room.created_by == auth.id,
                        room_code: room.room_code,
                        user_count,
                    };

                    return Ok(HttpResponse::Ok().json(JoinRoomByCodeResponse {
                        success: true,
                        message: "You are already a member of this room".to_string(),
                        room: Some(room_response),
                    }));
                },
                Ok(None) => {
                    let membership_id = Uuid::new_v4();
                    let membership = RoomMembershipActiveModel {
                        id: ActiveValue::Set(membership_id),
                        user_id: ActiveValue::Set(auth.id),
                        room_id: ActiveValue::Set(room.id),
                        joined_at: ActiveValue::Set(Utc::now()),
                    };
                    match RoomMembership::insert(membership).exec(db.get_ref()).await {
                        Ok(_) => {
                            // Get user count for this room (after joining)
                            let user_count_result = RoomMembership::find()
                                .filter(crate::models::entities::room_membership::Column::RoomId.eq(room.id))
                                .count(db.get_ref())
                                .await;
                            
                            let user_count = match user_count_result {
                                Ok(count) => count,
                                Err(e) => {
                                    log::error!("Failed to get user count for room {}: {}", room.id, e);
                                    0
                                }
                            };
                            
                            let room_response = RoomResponseDto {
                                id: room.id,
                                name: room.name,
                                description: room.description,
                                created_by: room.created_by,
                                created_at: room.created_at,
                                is_protected: room.password_hash.is_some(),
                                is_owner: room.created_by == auth.id,
                                room_code: room.room_code,
                                user_count,
                            };
                            Ok(HttpResponse::Ok().json(JoinRoomByCodeResponse {
                                success: true,
                                message: "Successfully joined the room".to_string(),
                                room: Some(room_response),
                            }))
                        },
                        Err(e) => {
                            log::error!("Failed to create room membership: {}", e);
                            Ok(HttpResponse::BadRequest().json(JoinRoomByCodeResponse {
                                success: false,
                                message: "Failed to join room due to invalid user or room reference".to_string(),
                                room: None,
                            }))
                        }
                    }
                },
                Err(e) => {
                    log::error!("Failed to check room membership: {}", e);
                    Ok(HttpResponse::InternalServerError().json(JoinRoomByCodeResponse {
                        success: false,
                        message: "Failed to check room membership".to_string(),
                        room: None,
                    }))
                }
            }
        },
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(JoinRoomByCodeResponse {
                success: false,
                message: "Room not found with the provided code".to_string(),
                room: None,
            }))
        },
        Err(e) => {
            log::error!("Failed to find room by code: {}", e);
            Ok(HttpResponse::InternalServerError().json(JoinRoomByCodeResponse {
                success: false,
                message: "Failed to find room".to_string(),
                room: None,
            }))
        }
    }
}