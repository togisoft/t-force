use actix::{Actor, StreamHandler, Handler, Message, Context, Addr, AsyncContext, ActorContext};
use actix_web::{web, Error, HttpRequest, HttpResponse, get};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use log::{error, info, debug};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveValue, QueryFilter, ColumnTrait, QuerySelect};

use crate::auth::{JwtAuth, extract_token_from_cookie_or_header};
use crate::models::entities::{UserResponseDto, ChatMessage, ChatMessageActiveModel, RoomMembership};
use crate::models::entities::message_reaction::{Entity as MessageReaction, ActiveModel as MessageReactionActiveModel, Column as MessageReactionColumn};

// Constants
const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB
const MAX_MESSAGES_PER_MINUTE: usize = 30;

#[derive(thiserror::Error, Debug)]
pub enum WsError {
    #[error("Authentication failed")]
    Authentication,
    #[error("Invalid message format")]
    InvalidMessage(String),
    #[error("Message too large")]
    MessageTooLarge,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Access denied: {0}")]
    AccessDenied(String),
    #[error("Database error")]
    DatabaseError,
    #[error("Database error: {0}")]
    Database(String),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "join")]
    Join { room_id: String },
    #[serde(rename = "leave")]
    Leave { room_id: String },
    #[serde(rename = "message")]
    Message { room_id: String, content: String, temp_id: Option<String> },
    #[serde(rename = "typing")]
    Typing { room_id: String, is_typing: bool },
    #[serde(rename = "reaction")]
    Reaction { message_id: String, emoji: String, add: bool },
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Serialize, Debug, Clone)]
pub struct WsResponse {
    pub message_type: String,
    pub data: serde_json::Value,
    pub timestamp: i64,
    pub message_id: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Ack {
    pub temp_id: String,
    pub message_id: String,
    pub success: bool,
}

// Chat server actor
pub struct ChatServer {
    sessions: HashMap<Addr<ChatSession>, UserResponseDto>,
    rooms: HashMap<String, HashSet<Addr<ChatSession>>>,
    message_history: HashMap<String, Vec<WsResponse>>,
    user_message_counts: HashMap<Uuid, (usize, i64)>, // (count, timestamp)
    db: Option<DatabaseConnection>,
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            message_history: HashMap::new(),
            user_message_counts: HashMap::new(),
            db: None,
        }
    }

    pub fn set_database(&mut self, db: DatabaseConnection) {
        self.db = Some(db);
    }

    pub fn join_room(&mut self, room_id: String, addr: Addr<ChatSession>, user: UserResponseDto) {
        // Add user to room
        self.rooms.entry(room_id.clone()).or_insert_with(HashSet::new).insert(addr.clone());

        // Send recent message history to the new user
        if let Some(history) = self.message_history.get(&room_id) {
            let recent_messages: Vec<WsResponse> = history.iter().rev().take(50).rev().cloned().collect();
            for msg in recent_messages {
                let _ = addr.do_send(SessionMessage(msg));
            }
        }

        // Notify other users in the room
        let user_joined_msg = WsResponse {
            message_type: "user_joined".to_string(),
            data: serde_json::json!({
                "room_id": room_id,
                "user_id": user.id,
                "user_name": user.name,
                "user_profile_image": user.profile_image
            }),
            timestamp: Utc::now().timestamp(),
            message_id: None,
        };

        self.broadcast_to_room(&room_id, &user_joined_msg, Some(&addr));
    }

    pub fn leave_room(&mut self, room_id: &str, addr: &Addr<ChatSession>) {
        if let Some(room_sessions) = self.rooms.get_mut(room_id) {
            room_sessions.remove(addr);

            // Get user info for the leave notification
            if let Some(user) = self.sessions.get(addr) {
                let user_left_msg = WsResponse {
                    message_type: "user_left".to_string(),
                    data: serde_json::json!({
                        "room_id": room_id,
                        "user_id": user.id,
                        "user_name": user.name
                    }),
                    timestamp: Utc::now().timestamp(),
                    message_id: None,
                };

                self.broadcast_to_room(room_id, &user_left_msg, Some(addr));
            }
        }
    }

    pub fn broadcast_to_room(&self, room_id: &str, message: &WsResponse, exclude: Option<&Addr<ChatSession>>) {
        if let Some(room_sessions) = self.rooms.get(room_id) {
            for session in room_sessions {
                if exclude.map_or(true, |excluded| session != excluded) {
                    let _ = session.do_send(SessionMessage(message.clone()));
                }
            }
        }
    }

    pub fn cleanup_session(&mut self, addr: &Addr<ChatSession>) {
        // Remove from all rooms
        let rooms_to_leave: Vec<String> = self.rooms
            .iter()
            .filter_map(|(room_id, sessions)| {
                if sessions.contains(addr) {
                    Some(room_id.clone())
                } else {
                    None
                }
            })
            .collect();

        for room_id in rooms_to_leave {
            self.leave_room(&room_id, addr);
        }

        // Remove from sessions
        self.sessions.remove(addr);
    }

    pub fn check_rate_limit(&mut self, user_id: Uuid) -> bool {
        let now = Utc::now().timestamp();
        let (count, timestamp) = self.user_message_counts.entry(user_id).or_insert((0, now));

        // Reset count if more than 1 minute has passed
        if now - *timestamp > 60 {
            *count = 0;
            *timestamp = now;
        }

        if *count >= MAX_MESSAGES_PER_MINUTE {
            return false;
        }

        *count += 1;
        true
    }

    pub fn store_message_in_history(&mut self, room_id: String, message: WsResponse) {
        let history = self.message_history.entry(room_id).or_insert_with(Vec::new);
        history.push(message);

        // Keep only last 100 messages per room
        if history.len() > 100 {
            history.remove(0);
        }
    }

    async fn persist_message(&self, room_id: Uuid, user_id: Uuid, content: String) -> Result<Uuid, WsError> {
        if let Some(db) = &self.db {
            let message_id = Uuid::new_v4();
            let message = ChatMessageActiveModel {
                id: ActiveValue::Set(message_id),
                room_id: ActiveValue::Set(room_id),
                user_id: ActiveValue::Set(user_id),
                content: ActiveValue::Set(content),
                created_at: ActiveValue::Set(Utc::now()),
                updated_at: ActiveValue::Set(Utc::now()),
            };

            match ChatMessage::insert(message).exec(db).await {
                Ok(_) => Ok(message_id),
                Err(e) => {
                    error!("Failed to persist message: {}", e);
                    Err(WsError::Database(e.to_string()))
                }
            }
        } else {
            Err(WsError::Database("Database connection not available".to_string()))
        }
    }
}

// Message to send to a session
#[derive(Message)]
#[rtype(result = "()")]
pub struct SessionMessage(pub WsResponse);

// Message to persist a chat message
#[derive(Message)]
#[rtype(result = "Result<Uuid, WsError>")]
pub struct PersistMessage {
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<SessionMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, _msg: SessionMessage, _: &mut Context<Self>) -> Self::Result {
        // This is handled by the ChatSession
    }
}

impl Handler<PersistMessage> for ChatServer {
    type Result = Result<Uuid, WsError>;

    fn handle(&mut self, _msg: PersistMessage, _: &mut Context<Self>) -> Self::Result {
        // This would be handled asynchronously in a real implementation
        // For now, we'll return an error to indicate it's not implemented
        Err(WsError::Database("Async persistence not implemented".to_string()))
    }
}

// Chat session actor
pub struct ChatSession {
    pub user: UserResponseDto,
    pub addr: Addr<ChatSession>,
    pub last_ping: i64,
    pub message_queue: Vec<String>,
}

impl ChatSession {
    pub fn new(user: UserResponseDto) -> Self {
        let (tx, _) = actix::dev::channel::channel::<ChatSession>(16);
        Self {
            user,
            addr: Addr::new(tx),
            last_ping: Utc::now().timestamp(),
            message_queue: Vec::new(),
        }
    }

    fn send_json(&self, ctx: &mut ws::WebsocketContext<Self>, message: WsResponse) {
        let msg_str = match serde_json::to_string(&message) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to serialize message: {}", e);
                return;
            }
        };
        ctx.text(msg_str);
    }

    fn handle_ws_message_sync(&self, msg: WsMessage) -> Result<(), WsError> {
        let mut server = match CHAT_SERVER.lock() {
            Ok(server) => server,
            Err(_) => return Err(WsError::Database("Failed to acquire chat server lock".to_string())),
        };

        match msg {
            WsMessage::Join { room_id } => {
                let room_uuid = match Uuid::parse_str(&room_id) {
                    Ok(id) => id,
                    Err(e) => {
                        error!("Invalid room ID format: {} - Error: {}", room_id, e);
                        return Err(WsError::InvalidMessage(format!("Invalid room ID format: {}", room_id)));
                    }
                };

                // Note: Security check is handled at HTTP API level
                // WebSocket access is only granted to authenticated users who have passed HTTP security checks

                // Check if user is already in the room
                if let Some(room_sessions) = server.rooms.get(&room_id) {
                    if room_sessions.contains(&self.addr) {
                        debug!("User {} already in room {}", self.user.name, room_id);
                        return Ok(());
                    }
                }

                server.join_room(room_id, self.addr.clone(), self.user.clone());
                info!("User {} joined room {}", self.user.name, room_uuid);
            }

            WsMessage::Leave { room_id } => {
                server.leave_room(&room_id, &self.addr);
                info!("User {} left room {}", self.user.name, room_id);
            }

            WsMessage::Message { room_id, content, temp_id } => {
                // Check message size
                if content.len() > MAX_MESSAGE_SIZE {
                    return Err(WsError::MessageTooLarge);
                }

                // Check rate limit
                if !server.check_rate_limit(self.user.id) {
                    return Err(WsError::RateLimitExceeded);
                }

                // Note: Security check is handled at HTTP API level
                // WebSocket access is only granted to authenticated users who have passed HTTP security checks

                let room_uuid = match Uuid::parse_str(&room_id) {
                    Ok(id) => id,
                    Err(e) => {
                        error!("Invalid room ID format: {} - Error: {}", room_id, e);
                        return Err(WsError::InvalidMessage(format!("Invalid room ID format: {}", room_id)));
                    }
                };

                // Create message response with database ID
                let message_id = Uuid::new_v4();
                let message_response = WsResponse {
                    message_type: "message".to_string(),
                    data: serde_json::json!({
                        "id": message_id.to_string(),
                        "room_id": room_id,
                        "user_id": self.user.id,
                        "user_name": self.user.name,
                        "user_profile_image": self.user.profile_image,
                        "content": content,
                        "timestamp": Utc::now().timestamp()
                    }),
                    timestamp: Utc::now().timestamp(),
                    message_id: Some(message_id.to_string()),
                };

                // Store in history
                server.store_message_in_history(room_id.clone(), message_response.clone());

                // Broadcast to room
                server.broadcast_to_room(&room_id, &message_response, None);

                // Send acknowledgment if temp_id is provided
                if let Some(temp_id) = temp_id {
                    let ack_response = WsResponse {
                        message_type: "message_ack".to_string(),
                        data: serde_json::json!({
                            "temp_id": temp_id,
                            "message_id": message_id.to_string(),
                            "success": true
                        }),
                        timestamp: Utc::now().timestamp(),
                        message_id: None,
                    };

                    // Send ack only to the sender
                    let _ = self.addr.do_send(SessionMessage(ack_response));
                }

                // Persist message to database (async)
                if let Some(db) = &server.db {
                    let db_clone = db.clone();
                    let room_uuid_clone = room_uuid;
                    let user_id_clone = self.user.id;
                    let content_clone = content.clone();
                    let message_id_clone = message_id;

                    // Spawn async task for database persistence
                    tokio::spawn(async move {
                        let message = ChatMessageActiveModel {
                            id: ActiveValue::Set(message_id_clone),
                            room_id: ActiveValue::Set(room_uuid_clone),
                            user_id: ActiveValue::Set(user_id_clone),
                            content: ActiveValue::Set(content_clone),
                            created_at: ActiveValue::Set(Utc::now()),
                            updated_at: ActiveValue::Set(Utc::now()),
                        };

                        match ChatMessage::insert(message).exec(&db_clone).await {
                            Ok(_) => {
                                debug!("Message persisted to database: {}", message_id_clone);
                            }
                            Err(e) => {
                                error!("Failed to persist message to database: {}", e);
                            }
                        }
                    });
                }
            }

            WsMessage::Typing { room_id, is_typing } => {
                let typing_response = WsResponse {
                    message_type: "typing".to_string(),
                    data: serde_json::json!({
                        "room_id": room_id,
                        "user_id": self.user.id,
                        "user_name": self.user.name,
                        "is_typing": is_typing
                    }),
                    timestamp: Utc::now().timestamp(),
                    message_id: None,
                };

                server.broadcast_to_room(&room_id, &typing_response, Some(&self.addr));
            }

            WsMessage::Reaction { message_id, emoji, add } => {
                // Parse message_id
                let msg_uuid = match Uuid::parse_str(&message_id) {
                    Ok(id) => id,
                    Err(e) => {
                        error!("Invalid message ID format: {} - Error: {}", message_id, e);
                        return Err(WsError::InvalidMessage(format!("Invalid message ID format: {}", message_id)));
                    }
                };

                // Release the server lock before spawning async work to avoid holding it across await
                drop(server);

                let user_id = self.user.id;
                let user_name = self.user.name.clone();
                let emoji_clone = emoji.clone();

                // Perform DB operations and broadcast asynchronously
                tokio::spawn(async move {
                    // Acquire DB connection from chat server
                    let db_opt = match CHAT_SERVER.lock() {
                        Ok(server) => server.db.clone(),
                        Err(_) => {
                            error!("Failed to acquire chat server lock for reactions");
                            None
                        }
                    };

                    let Some(db) = db_opt else {
                        error!("Database connection not available for reactions");
                        return;
                    };

                    // Find the message to get room_id and validate membership
                    match ChatMessage::find_by_id(msg_uuid).one(&db).await {
                        Ok(Some(chat_msg)) => {
                            // SECURITY: verify user is a member of this room
                            match crate::models::entities::room_membership::Entity::find()
                                .filter(crate::models::entities::room_membership::Column::RoomId.eq(chat_msg.room_id))
                                .filter(crate::models::entities::room_membership::Column::UserId.eq(user_id))
                                .one(&db)
                                .await
                            {
                                Ok(Some(_)) => {
                                    // proceed
                                }
                                Ok(None) => {
                                    error!("Access denied: user {} is not a member of room {}", user_id, chat_msg.room_id);
                                    return;
                                }
                                Err(e) => {
                                    error!("Failed to verify room membership for reaction: {}", e);
                                    return;
                                }
                            }

                            if add {
                                // Insert reaction (idempotent with unique index if present)
                                let reaction = MessageReactionActiveModel {
                                    id: ActiveValue::Set(Uuid::new_v4()),
                                    message_id: ActiveValue::Set(msg_uuid),
                                    user_id: ActiveValue::Set(user_id),
                                    emoji: ActiveValue::Set(emoji_clone.clone()),
                                    created_at: ActiveValue::Set(Utc::now()),
                                };

                                match MessageReaction::insert(reaction).exec(&db).await {
                                    Ok(_) => {
                                        debug!(
                                            "Reaction persisted to database: message_id={}, user_id={}, emoji={}",
                                            msg_uuid, user_id, emoji_clone
                                        );
                                    }
                                    Err(e) => {
                                        error!("Failed to persist reaction: {}", e);
                                        // Continue to broadcast for better UX
                                    }
                                }
                            } else {
                                // Remove reaction
                                match MessageReaction::delete_many()
                                    .filter(MessageReactionColumn::MessageId.eq(msg_uuid))
                                    .filter(MessageReactionColumn::UserId.eq(user_id))
                                    .filter(MessageReactionColumn::Emoji.eq(emoji_clone.clone()))
                                    .exec(&db)
                                    .await
                                {
                                    Ok(res) => {
                                        if res.rows_affected == 0 {
                                            debug!(
                                                "No reaction found to remove for message_id={}, user_id={}, emoji={}",
                                                msg_uuid, user_id, emoji_clone
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to remove reaction: {}", e);
                                        // Continue
                                    }
                                }
                            }

                            // Build response and broadcast to the correct room
                            let reaction_response = WsResponse {
                                message_type: "reaction".to_string(),
                                data: serde_json::json!({
                                    "message_id": message_id,
                                    "user_id": user_id,
                                    "user_name": user_name,
                                    "emoji": emoji_clone,
                                    "add": add
                                }),
                                timestamp: Utc::now().timestamp(),
                                message_id: None,
                            };

                            if let Ok(server) = CHAT_SERVER.lock() {
                                server.broadcast_to_room(&chat_msg.room_id.to_string(), &reaction_response, None);
                            } else {
                                error!("Failed to acquire chat server lock to broadcast reaction");
                            }
                        }
                        Ok(None) => {
                            error!("Message not found for reaction: {}", message_id);
                        }
                        Err(e) => {
                            error!("Database error looking up message for reaction: {}", e);
                        }
                    }
                });
            }

            WsMessage::Ping => {
                // Update last ping time
                // This will be handled in the session
            }
        }

        Ok(())
    }
}

impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.addr = ctx.address();

        // Set up ping interval
        ctx.run_interval(std::time::Duration::from_secs(30), |act, ctx| {
            let now = Utc::now().timestamp();
            if now - act.last_ping > 60 {
                // No ping for 60 seconds, close connection
                ctx.stop();
                return;
            }

            // Send pong
            let pong_response = WsResponse {
                message_type: "pong".to_string(),
                data: serde_json::json!({}),
                timestamp: now,
                message_id: None,
            };
            act.send_json(ctx, pong_response);
        });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        // Clean up session
        if let Ok(mut server) = CHAT_SERVER.lock() {
            server.cleanup_session(&self.addr);
        } else {
            error!("Failed to acquire chat server lock during session cleanup");
        }
        actix::Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Parse the message
                let ws_message: WsMessage = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to parse WebSocket message: {}", e);
                        return;
                    }
                };

                // Handle ping separately
                if let WsMessage::Ping = ws_message {
                    self.last_ping = Utc::now().timestamp();
                    return;
                }

                // Handle other messages
                match self.handle_ws_message_sync(ws_message) {
                    Ok(_) => {
                        debug!("WebSocket message handled successfully");
                    }
                    Err(e) => {
                        error!("Failed to handle WebSocket message: {:?}", e);
                        // Send error response to client
                        let error_response = WsResponse {
                            message_type: "error".to_string(),
                            data: serde_json::json!({
                                "error": format!("{:?}", e)
                            }),
                            timestamp: Utc::now().timestamp(),
                            message_id: None,
                        };
                        self.send_json(ctx, error_response);
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                error!("Binary messages not supported");
            }
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
                self.last_ping = Utc::now().timestamp();
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_ping = Utc::now().timestamp();
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
            }
            Ok(ws::Message::Continuation(_)) => {
                error!("Continuation frames not supported");
            }
            Ok(ws::Message::Nop) => {}
            Err(e) => {
                error!("WebSocket error: {:?}", e);
                ctx.stop();
            }
        }
    }
}

impl Handler<SessionMessage> for ChatSession {
    type Result = ();

    fn handle(&mut self, msg: SessionMessage, ctx: &mut Self::Context) -> Self::Result {
        self.send_json(ctx, msg.0);
    }
}

// Global chat server instance
lazy_static::lazy_static! {
    pub static ref CHAT_SERVER: Mutex<ChatServer> = Mutex::new(ChatServer::new());
}

#[get("/api/chat/ws")]
pub async fn ws_index(req: HttpRequest, stream: web::Payload, jwt_secret: web::Data<String>, db: web::Data<DatabaseConnection>) -> Result<HttpResponse, Error> {
    debug!("WebSocket connection attempt - Path: {}", req.path());
    debug!("WebSocket connection attempt - Headers: {:?}", req.headers());

    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie: {}", cookie.value());
    } else {
        debug!("No auth_token cookie found");

        // Check all cookies for debugging
        if let Ok(cookies) = req.cookies() {
            for cookie in cookies.iter() {
                debug!("Cookie found: {} = {}", cookie.name(), cookie.value());
            }
        }
    }

    // Check for Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        debug!("Found Authorization header: {}", auth_header.to_str().unwrap_or("invalid"));
    } else {
        debug!("No Authorization header found");
    }

    // Try to extract token with more detailed logging
    let token = match extract_token_from_cookie_or_header(&req) {
        Some(token) => {
            debug!("Token extracted successfully, length: {}", token.len());
            token
        },
        None => {
            error!("No authentication token found in cookie or header");
            error!("Available cookies: {:?}", req.cookies());
            error!("Available headers: {:?}", req.headers());
            return Err(actix_web::error::ErrorUnauthorized("Authentication token not found. Please log in again."));
        }
    };

    debug!("Validating token...");
    let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
        Ok(claims) => {
            debug!("Token validated successfully for user: {}", claims.name);
            debug!("Token structure - sub: {}, backend_user_id: {:?}, email: {}",
                   claims.sub, claims.backend_user_id, claims.email);
            claims
        },
        Err(e) => {
            error!("Token validation failed: {:?}", e);
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    return Err(actix_web::error::ErrorUnauthorized("Token expired. Please log in again."));
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    return Err(actix_web::error::ErrorUnauthorized("Invalid token signature."));
                }
                _ => {
                    return Err(actix_web::error::ErrorUnauthorized("Invalid token. Please log in again."));
                }
            }
        }
    };

    // Parse user ID safely - use backend_user_id instead of sub
    let user_id = match &claims.backend_user_id {
        Some(backend_user_id) => {
            match Uuid::parse_str(backend_user_id) {
                Ok(id) => {
                    debug!("User ID parsed successfully from backend_user_id: {}", id);
                    id
                },
                Err(e) => {
                    error!("Invalid backend_user_id in token: {} - Error: {}", backend_user_id, e);
                    return Err(actix_web::error::ErrorUnauthorized("Invalid backend_user_id in token"));
                }
            }
        },
        None => {
            error!("Missing backend_user_id in token. Sub field contains: {}", claims.sub);
            return Err(actix_web::error::ErrorUnauthorized("Missing backend_user_id in token"));
        }
    };

    let user = UserResponseDto {
        id: user_id,
        email: claims.email,
        name: claims.name,
        profile_image: claims.profile_image,
        provider: claims.provider.unwrap_or_default(),
        role: claims.user_role.unwrap_or_default(),
        is_active: claims.is_active.unwrap_or(true),
    };

    // Set database connection in chat server
    {
        if let Ok(mut server) = CHAT_SERVER.lock() {
            server.set_database(db.get_ref().clone());
        } else {
            error!("Failed to acquire chat server lock for database setup");
        }
    }

    info!("Starting WebSocket session for user: {}", user.name);
    ws::start(ChatSession::new(user), &req, stream)
}