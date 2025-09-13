use actix_web::{web, HttpResponse, post, get, delete, Responder, HttpRequest, Error};
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveValue, QueryOrder, Order, QueryFilter, ColumnTrait, DeleteResult, PaginatorTrait};
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::auth::{AuthUser, JwtAuth, extract_token_from_cookie_or_header};
use crate::models::entities::{ChatRoom, ChatRoomActiveModel, CreateRoomDto, RoomResponseDto, RoomMembership, RoomMembershipActiveModel};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString, PasswordHash, PasswordVerifier
    },
    Argon2
};

#[derive(Deserialize)]
pub struct RoomPath {
    room_id: Uuid,
}

#[derive(Deserialize)]
pub struct VerifyRoomPasswordRequest {
    password: String,
}

#[derive(Deserialize)]
pub struct JoinRoomByCodeRequest {
    room_code: String,
}

#[derive(Serialize)]
pub struct VerifyRoomPasswordResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
pub struct PasswordProtectedResponse {
    is_protected: bool,
    message: String,
}

#[derive(Serialize)]
pub struct DeleteRoomResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
pub struct LeaveRoomResponse {
    success: bool,
    message: String,
}

#[post("/api/chat/rooms")]
pub async fn create_room(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    room_data: web::Json<CreateRoomDto>,
) -> Result<HttpResponse, Error> {
    // Extract token from request
    let token = match extract_token_from_cookie_or_header(&req) {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: No authentication token found"
            })));
        }
    };

    // Validate token and get user
    let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
        Ok(claims) => claims,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid authentication token"
            })));
        }
    };

    // Ensure backend_user_id is present and valid
    let backend_user_id = match claims.backend_user_id {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Missing user ID in token"
            })));
        }
    };

    let user_id = match Uuid::parse_str(&backend_user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid user ID format"
            })));
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

    let room_id = Uuid::new_v4();
    
    // Generate a room code (first 6 characters of the UUID)
    let room_code = room_id.to_string().chars().take(6).collect::<String>().to_uppercase();
    
    // Hash password if provided
    let has_password = if let Some(password) = &room_data.password {
        !password.is_empty()
    } else {
        false
    };

    let password_hash = if has_password {
        if let Some(password) = &room_data.password {
            // Generate a salt
            let salt = SaltString::generate(&mut OsRng);
            
            // Hash the password
            let argon2 = Argon2::default();
            match argon2.hash_password(password.as_bytes(), &salt) {
                Ok(hash) => Some(hash.to_string()),
                Err(e) => {
                    log::error!("Failed to hash room password: {}", e);
                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to create room: password hashing error"
                    })));
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    let room = ChatRoomActiveModel {
        id: ActiveValue::Set(room_id),
        name: ActiveValue::Set(room_data.name.clone()),
        description: ActiveValue::Set(room_data.description.clone()),
        created_by: ActiveValue::Set(auth.id),
        created_at: ActiveValue::Set(Utc::now()),
        updated_at: ActiveValue::Set(Utc::now()),
        password_hash: ActiveValue::Set(password_hash),
        room_code: ActiveValue::Set(room_code.clone()),
    };

    match ChatRoom::insert(room).exec(db.get_ref()).await {
        Ok(_) => {
            // Add creator as a member of the room
            let membership_id = Uuid::new_v4();
            let creator_membership = RoomMembershipActiveModel {
                id: ActiveValue::Set(membership_id),
                user_id: ActiveValue::Set(auth.id),
                room_id: ActiveValue::Set(room_id),
                joined_at: ActiveValue::Set(Utc::now()),
            };
            
            match RoomMembership::insert(creator_membership).exec(db.get_ref()).await {
                Ok(_) => {
                    let room_response = RoomResponseDto {
                        id: room_id,
                        name: room_data.name.clone(),
                        description: room_data.description.clone(),
                        created_by: auth.id,
                        created_at: Utc::now(),
                        is_protected: has_password,
                        is_owner: true, // Creator is always the owner
                        room_code: room_code,
                        user_count: 1, // Creator is the first member
                    };

                    Ok(HttpResponse::Created().json(room_response))
                }
                Err(e) => {
                    log::error!("Failed to add creator as member: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to create room: could not add creator as member"
                    })))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create room: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create room"
            })))
        }
    }
}

#[get("/api/chat/rooms")]
pub async fn get_rooms(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
) -> Result<HttpResponse, Error> {
    // Extract token from request
    let token = match extract_token_from_cookie_or_header(&req) {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: No authentication token found"
            })));
        }
    };

    // Validate token and get user
    let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
        Ok(claims) => claims,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid authentication token"
            })));
        }
    };

    // Ensure backend_user_id is present and valid
    let backend_user_id = match claims.backend_user_id {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Missing user ID in token"
            })));
        }
    };

    let user_id = match Uuid::parse_str(&backend_user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid user ID format"
            })));
        }
    };

    // Get rooms that the user created or joined
    use sea_orm::QuerySelect;
    
    // First, get all rooms created by the user
    let created_rooms_result = ChatRoom::find()
        .filter(crate::models::entities::chat_room::Column::CreatedBy.eq(user_id))
        .all(db.get_ref())
        .await;
    
    // Then, get all rooms the user has joined
    let joined_rooms_result = RoomMembership::find()
        .filter(crate::models::entities::room_membership::Column::UserId.eq(user_id))
        .find_with_related(ChatRoom)
        .all(db.get_ref())
        .await;
    
    // Handle errors for both queries
    let created_rooms = match created_rooms_result {
        Ok(rooms) => rooms,
        Err(e) => {
            log::error!("Failed to get created rooms: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get created rooms"
            })));
        }
    };
    
    let joined_rooms = match joined_rooms_result {
        Ok(rooms) => rooms,
        Err(e) => {
            log::error!("Failed to get joined rooms: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get joined rooms"
            })));
        }
    };
    
    // Combine the two lists, removing duplicates
    let mut all_rooms = created_rooms;
    
    // Add joined rooms that weren't created by the user
    for (_, rooms) in joined_rooms {
        for room in rooms {
            if !all_rooms.iter().any(|r| r.id == room.id) {
                all_rooms.push(room);
            }
        }
    }
    
    // Sort by creation date (newest first)
    all_rooms.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Get user count for each room
    let mut room_responses: Vec<RoomResponseDto> = Vec::new();
    
    for room in all_rooms {
        // Count members for this room
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
        
        room_responses.push(RoomResponseDto {
            id: room.id,
            name: room.name,
            description: room.description,
            created_by: room.created_by,
            created_at: room.created_at,
            is_protected: room.password_hash.is_some(),
            is_owner: room.created_by == user_id,
            room_code: room.room_code,
            user_count,
        });
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "rooms": room_responses
    })))
}

#[post("/api/chat/rooms/{room_id}/verify-password")]
pub async fn verify_room_password(
    db: web::Data<DatabaseConnection>,
    http_req: HttpRequest,
    jwt_secret: web::Data<String>,
    path: web::Path<RoomPath>,
    password_req: web::Json<VerifyRoomPasswordRequest>,
) -> Result<HttpResponse, Error> {
    // Extract token from request
    let token = match extract_token_from_cookie_or_header(&http_req) {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: No authentication token found"
            })));
        }
    };

    // Validate token and get user
    let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
        Ok(claims) => claims,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid authentication token"
            })));
        }
    };

    // Ensure backend_user_id is present and valid
    let backend_user_id = match claims.backend_user_id {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Missing user ID in token"
            })));
        }
    };

    let user_id = match Uuid::parse_str(&backend_user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid user ID format"
            })));
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

    let room_id = path.room_id;

    // Find the room
    match ChatRoom::find_by_id(room_id).one(db.get_ref()).await {
        Ok(Some(room)) => {
            // Check if room has a password
            if let Some(password_hash) = room.password_hash {
                // Verify the password
                let parsed_hash = match PasswordHash::new(&password_hash) {
                    Ok(hash) => hash,
                    Err(e) => {
                        log::error!("Failed to parse stored password hash: {:?}", e);
                        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Internal server error"
                        })));
                    }
                };

                if Argon2::default().verify_password(password_req.password.as_bytes(), &parsed_hash).is_ok() {
                    // Password is correct
                    Ok(HttpResponse::Ok().json(VerifyRoomPasswordResponse {
                        success: true,
                        message: "Password verified successfully".to_string(),
                    }))
                } else {
                    // Password is incorrect
                    Ok(HttpResponse::Unauthorized().json(VerifyRoomPasswordResponse {
                        success: false,
                        message: "Incorrect password".to_string(),
                    }))
                }
            } else {
                // Room doesn't have a password
                Ok(HttpResponse::BadRequest().json(VerifyRoomPasswordResponse {
                    success: false,
                    message: "Room is not password protected".to_string(),
                }))
            }
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Room not found"
            })))
        }
        Err(e) => {
            log::error!("Failed to get room: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get room"
            })))
        }
    }
}

#[get("/api/chat/rooms/{room_id}")]
pub async fn get_room(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    path: web::Path<RoomPath>,
) -> Result<HttpResponse, Error> {
    // Extract token from request
    let token = match extract_token_from_cookie_or_header(&req) {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: No authentication token found"
            })));
        }
    };

    // Validate token and get user
    let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
        Ok(claims) => claims,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid authentication token"
            })));
        }
    };

    // Ensure backend_user_id is present and valid
    let backend_user_id = match claims.backend_user_id {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Missing user ID in token"
            })));
        }
    };

    let user_id = match Uuid::parse_str(&backend_user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid user ID format"
            })));
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

    let room_id = path.room_id;

    match ChatRoom::find_by_id(room_id)
        .one(db.get_ref())
        .await {
        Ok(Some(room)) => {
            // Check if room is password protected
            if room.password_hash.is_some() {
                // Return a special response for password-protected rooms
                return Ok(HttpResponse::Forbidden().json(PasswordProtectedResponse {
                    is_protected: true,
                    message: "This room is password protected. Please verify your password.".to_string(),
                }));
            }
            
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
                is_protected: false, // We already checked and it's not protected
                is_owner: room.created_by == user_id,
                room_code: room.room_code,
                user_count,
            };

            Ok(HttpResponse::Ok().json(room_response))
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Room not found"
            })))
        }
        Err(e) => {
            log::error!("Failed to get room: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get room"
            })))
        }
    }
}

#[delete("/api/chat/rooms/{room_id}")]
pub async fn delete_room(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    path: web::Path<RoomPath>,
) -> Result<HttpResponse, Error> {
    // Extract token from request
    let token = match extract_token_from_cookie_or_header(&req) {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: No authentication token found"
            })));
        }
    };

    // Validate token and get user
    let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
        Ok(claims) => claims,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid authentication token"
            })));
        }
    };

    // Ensure backend_user_id is present and valid
    let backend_user_id = match claims.backend_user_id {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Missing user ID in token"
            })));
        }
    };

    let user_id = match Uuid::parse_str(&backend_user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid user ID format"
            })));
        }
    };

    let room_id = path.room_id;

    // Find the room
    match ChatRoom::find_by_id(room_id).one(db.get_ref()).await {
        Ok(Some(room)) => {
            // Check if the user is the creator of the room
            if room.created_by != user_id {
                return Ok(HttpResponse::Forbidden().json(DeleteRoomResponse {
                    success: false,
                    message: "You can only delete rooms that you have created".to_string(),
                }));
            }

            // Delete the room
            match ChatRoom::delete_by_id(room_id).exec(db.get_ref()).await {
                Ok(_) => {
                    Ok(HttpResponse::Ok().json(DeleteRoomResponse {
                        success: true,
                        message: "Room deleted successfully".to_string(),
                    }))
                }
                Err(e) => {
                    log::error!("Failed to delete room: {}", e);
                    Ok(HttpResponse::InternalServerError().json(DeleteRoomResponse {
                        success: false,
                        message: "Failed to delete room".to_string(),
                    }))
                }
            }
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(DeleteRoomResponse {
                success: false,
                message: "Room not found".to_string(),
            }))
        }
        Err(e) => {
            log::error!("Failed to get room: {}", e);
            Ok(HttpResponse::InternalServerError().json(DeleteRoomResponse {
                success: false,
                message: "Failed to get room".to_string(),
            }))
        }
    }
}

#[delete("/api/chat/rooms/{room_id}/membership")]
pub async fn leave_room_membership(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    path: web::Path<RoomPath>,
) -> Result<HttpResponse, Error> {
    // Extract token
    let token = match extract_token_from_cookie_or_header(&req) {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: No authentication token found"
            })));
        }
    };

    // Validate token
    let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
        Ok(claims) => claims,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid authentication token"
            })));
        }
    };

    // Extract user id
    let backend_user_id = match claims.backend_user_id {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Missing user ID in token"
            })));
        }
    };

    let user_id = match Uuid::parse_str(&backend_user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Invalid user ID format"
            })));
        }
    };

    let room_id = path.room_id;

    // Ensure room exists and the user is not the owner (owners can't leave their own rooms via membership deletion)
    match ChatRoom::find_by_id(room_id).one(db.get_ref()).await {
        Ok(Some(room)) => {
            if room.created_by == user_id {
                return Ok(HttpResponse::Forbidden().json(LeaveRoomResponse {
                    success: false,
                    message: "Room owners cannot leave their own room".to_string(),
                }));
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(LeaveRoomResponse {
                success: false,
                message: "Room not found".to_string(),
            }));
        }
        Err(e) => {
            log::error!("Failed to get room: {}", e);
            return Ok(HttpResponse::InternalServerError().json(LeaveRoomResponse {
                success: false,
                message: "Failed to process request".to_string(),
            }));
        }
    }

    // Delete membership
    use sea_orm::EntityTrait;
    use sea_orm::ColumnTrait;
    let delete_result = RoomMembership::delete_many()
        .filter(crate::models::entities::room_membership::Column::UserId.eq(user_id))
        .filter(crate::models::entities::room_membership::Column::RoomId.eq(room_id))
        .exec(db.get_ref())
        .await;

    match delete_result {
        Ok(result) => {
            if result.rows_affected > 0 {
                Ok(HttpResponse::Ok().json(LeaveRoomResponse {
                    success: true,
                    message: "Left room successfully".to_string(),
                }))
            } else {
                Ok(HttpResponse::NotFound().json(LeaveRoomResponse {
                    success: false,
                    message: "Membership not found".to_string(),
                }))
            }
        }
        Err(e) => {
            log::error!("Failed to delete room membership: {}", e);
            Ok(HttpResponse::InternalServerError().json(LeaveRoomResponse {
                success: false,
                message: "Failed to leave room".to_string(),
            }))
        }
    }
}