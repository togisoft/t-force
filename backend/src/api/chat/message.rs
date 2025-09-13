use actix_web::{web, HttpResponse, post, get, Responder, HttpRequest, Error};
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveValue, QueryOrder, Order, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::auth::{AuthUser, JwtAuth, extract_token_from_cookie_or_header};
use crate::models::entities::{ChatMessage, ChatMessageActiveModel, CreateMessageDto, MessageResponseDto, MessageWithUserDto, User, RoomMembership, MessageReaction};
use std::collections::{HashMap, HashSet};

#[derive(Deserialize)]
pub struct MessagePath {
    room_id: Uuid,
}

#[derive(Deserialize)]
pub struct MessageQuery {
    limit: Option<u64>,
    offset: Option<u64>,
}

#[post("/api/chat/messages")]
pub async fn send_message(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    message_data: web::Json<CreateMessageDto>,
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

    let message_id = Uuid::new_v4();

    // SECURITY CHECK: Verify user is a member of this room
    let membership_check = RoomMembership::find()
        .filter(crate::models::entities::room_membership::Column::RoomId.eq(message_data.room_id))
        .filter(crate::models::entities::room_membership::Column::UserId.eq(auth.id))
        .one(db.get_ref())
        .await;

    match membership_check {
        Ok(Some(_)) => {
            // User is a member, proceed to send message
        }
        Ok(None) => {
            // User is not a member of this room
            return Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied: You are not a member of this room"
            })));
        }
        Err(e) => {
            log::error!("Failed to check room membership: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to verify room access"
            })));
        }
    }

    let message = ChatMessageActiveModel {
        id: ActiveValue::Set(message_id),
        room_id: ActiveValue::Set(message_data.room_id),
        user_id: ActiveValue::Set(auth.id),
        content: ActiveValue::Set(message_data.content.clone()),
        created_at: ActiveValue::Set(Utc::now()),
        updated_at: ActiveValue::Set(Utc::now()),
    };

    // Debug: Log the message data before inserting
    log::info!("Attempting to insert message: room_id={}, user_id={}, content={}", 
        message_data.room_id, auth.id, message_data.content);

    match ChatMessage::insert(message).exec(db.get_ref()).await {
        Ok(result) => {
            log::info!("Successfully inserted message with result: {:?}", result);
            
            let message_response = MessageResponseDto {
                id: message_id,
                room_id: message_data.room_id,
                user_id: auth.id,
                content: message_data.content.clone(),
                created_at: Utc::now(),
                user_name: Some(auth.name.clone()),
            };

            Ok(HttpResponse::Created().json(message_response))
        }
        Err(e) => {
            log::error!("Failed to send message to database: {:?}", e);
            log::error!("Error details: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to send message: {}", e)
            })))
        }
    }
}

#[get("/api/chat/rooms/{room_id}/messages")]
pub async fn get_messages(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    path: web::Path<MessagePath>,
    query: web::Query<MessageQuery>,
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
    let limit = query.limit.unwrap_or(200);
    let offset = query.offset.unwrap_or(0);

    // SECURITY CHECK: Verify user is a member of this room
    let membership_check = RoomMembership::find()
        .filter(crate::models::entities::room_membership::Column::RoomId.eq(room_id))
        .filter(crate::models::entities::room_membership::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await;

    match membership_check {
        Ok(Some(_)) => {
            // User is a member, proceed to get messages
        }
        Ok(None) => {
            // User is not a member of this room
            return Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied: You are not a member of this room"
            })));
        }
        Err(e) => {
            log::error!("Failed to check room membership: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to verify room access"
            })));
        }
    }

    let messages = ChatMessage::find()
        .filter(crate::models::entities::chat_message::Column::RoomId.eq(room_id))
        .order_by(crate::models::entities::chat_message::Column::CreatedAt, Order::Asc)
        .limit(limit)
        .offset(offset)
        .all(db.get_ref())
        .await;

    match messages {
        Ok(messages) => {
            let mut message_with_users = Vec::new();

            for message in messages {
                let user = User::find_by_id(message.user_id)
                    .one(db.get_ref())
                    .await;

                match user {
                    Ok(Some(user)) => {
                        message_with_users.push(MessageWithUserDto {
                            id: message.id,
                            room_id: message.room_id,
                            user_id: message.user_id,
                            user_name: user.name,
                            user_profile_image: user.profile_image,
                            content: message.content,
                            created_at: message.created_at,
                        });
                    }
                    _ => {
                        message_with_users.push(MessageWithUserDto {
                            id: message.id,
                            room_id: message.room_id,
                            user_id: message.user_id,
                            user_name: "Unknown User".to_string(),
                            user_profile_image: None,
                            content: message.content,
                            created_at: message.created_at,
                        });
                    }
                }
            }

            // Collect message IDs for reactions lookup
            let message_ids: Vec<Uuid> = message_with_users.iter().map(|m| m.id).collect();

            // Fetch reactions for all messages in batch
            let reactions = if message_ids.is_empty() {
                Vec::new()
            } else {
                match MessageReaction::find()
                    .filter(crate::models::entities::message_reaction::Column::MessageId.is_in(message_ids.clone()))
                    .all(db.get_ref())
                    .await {
                        Ok(r) => r,
                        Err(e) => {
                            log::error!("Failed to load reactions: {}", e);
                            Vec::new()
                        }
                    }
            };

            // Fetch all users involved in reactions to avoid N+1
            let mut reaction_user_ids: HashSet<Uuid> = HashSet::new();
            for r in &reactions { reaction_user_ids.insert(r.user_id); }

            let reaction_users = if reaction_user_ids.is_empty() {
                Vec::new()
            } else {
                match User::find()
                    .filter(crate::models::entities::user::Column::Id.is_in(reaction_user_ids.iter().cloned().collect::<Vec<_>>()))
                    .all(db.get_ref())
                    .await {
                        Ok(us) => us,
                        Err(e) => {
                            log::error!("Failed to load users for reactions: {}", e);
                            Vec::new()
                        }
                    }
            };

            let user_map: HashMap<Uuid, crate::models::entities::user::Model> =
                reaction_users.into_iter().map(|u| (u.id, u)).collect();

            // Group reactions by message and emoji
            let mut grouped: HashMap<Uuid, HashMap<String, Vec<serde_json::Value>>> = HashMap::new();
            for r in reactions {
                let emoji = r.emoji.clone();
                let msg_id = r.message_id;
                let user_info = if let Some(u) = user_map.get(&r.user_id) {
                    serde_json::json!({
                        "user_id": u.id,
                        "user_name": u.name,
                        "user_profile_image": u.profile_image
                    })
                } else {
                    serde_json::json!({
                        "user_id": r.user_id,
                        "user_name": "Unknown User",
                        "user_profile_image": null
                    })
                };
                grouped
                    .entry(msg_id)
                    .or_default()
                    .entry(emoji)
                    .or_default()
                    .push(user_info);
            }

            // Build enriched messages including reactions as expected by frontend
            let enriched_messages: Vec<serde_json::Value> = message_with_users
                .into_iter()
                .map(|m| {
                    let reactions_json: Vec<serde_json::Value> = if let Some(by_emoji) = grouped.get(&m.id) {
                        by_emoji.iter().map(|(emoji, users)| {
                            serde_json::json!({
                                "id": format!("{}:{}", m.id, emoji),
                                "emoji": emoji,
                                "count": users.len(),
                                "users": users
                            })
                        }).collect()
                    } else { Vec::new() };

                    serde_json::json!({
                        "id": m.id,
                        "room_id": m.room_id,
                        "user_id": m.user_id,
                        "user_name": m.user_name,
                        "user_profile_image": m.user_profile_image,
                        "content": m.content,
                        "created_at": m.created_at,
                        "reactions": reactions_json
                    })
                })
                .collect();

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "messages": enriched_messages
            })))
        }
        Err(e) => {
            log::error!("Failed to get messages: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get messages"
            })))
        }
    }
}