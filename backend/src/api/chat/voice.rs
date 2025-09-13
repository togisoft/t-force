use actix_web::{web, HttpResponse, post, get, HttpRequest, Error};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveValue, QueryFilter, ColumnTrait};
use uuid::Uuid;
use chrono::Utc;
use std::io::Write;
use std::path::Path;
use std::fs;
use crate::auth::{AuthUser, JwtAuth, extract_token_from_cookie_or_header};
use crate::models::entities::{ChatMessage, ChatMessageActiveModel};
use crate::api::chat::ws::{WsResponse, CHAT_SERVER};

#[post("/api/chat/voice")]
pub async fn upload_voice_message(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    mut payload: Multipart,
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

    let mut room_id: Option<Uuid> = None;
    let mut audio_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    // Process multipart form data
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        
        if let Some(content_disposition) = content_disposition {
            if let Some(name) = content_disposition.get_name() {
                let name_str = name.to_string();
                let filename_from_disposition = content_disposition.get_filename().map(|f| f.to_string());
                
                match name_str.as_str() {
                    "room_id" => {
                        let mut data = Vec::new();
                        while let Some(chunk) = field.next().await {
                            data.extend_from_slice(&chunk?);
                        }
                        let room_id_str = String::from_utf8(data).map_err(|e| {
                            log::error!("Failed to parse room_id as UTF-8: {}", e);
                            actix_web::error::ErrorBadRequest("Invalid room_id format")
                        })?;
                        room_id = Some(Uuid::parse_str(&room_id_str).map_err(|e| {
                            log::error!("Failed to parse room_id as UUID: {}", e);
                            actix_web::error::ErrorBadRequest("Invalid room_id format")
                        })?);
                    }
                    "audio" => {
                        let mut data = Vec::new();
                        while let Some(chunk) = field.next().await {
                            data.extend_from_slice(&chunk?);
                        }
                        audio_data = Some(data);
                        
                        // Get filename if available
                        if let Some(disposition) = filename_from_disposition {
                            filename = Some(disposition);
                        }
                    }
                    _ => {
                        // Skip unknown fields
                        while let Some(_) = field.next().await {}
                    }
                }
            }
        }
    }

    // Validate required fields
    let room_id = match room_id {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Missing room_id"
            })));
        }
    };

    let audio_data = match audio_data {
        Some(data) => data,
        None => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Missing audio file"
            })));
        }
    };

    // Validate file size (max 10MB)
    if audio_data.len() > 10 * 1024 * 1024 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Audio file too large. Maximum size is 10MB."
        })));
    }

    // Generate unique filename
    let file_extension = if let Some(ref name) = filename {
        Path::new(name).extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("webm")
    } else {
        "webm"
    };
    
    let unique_filename = format!("voice_{}_{}.{}", 
        Uuid::new_v4().to_string(), 
        Utc::now().timestamp(), 
        file_extension
    );

    // Create uploads directory if it doesn't exist
    let uploads_dir = Path::new("./uploads/voice_messages");
    if !uploads_dir.exists() {
        std::fs::create_dir_all(uploads_dir)?;
    }

    // Save audio file
    let file_path = uploads_dir.join(&unique_filename);
    let mut file = std::fs::File::create(&file_path)?;
    file.write_all(&audio_data)?;

    // Create message in database
    let message_id = Uuid::new_v4();
    let audio_url = format!("/api/chat/voice/{}", unique_filename);
    

    
    let message = ChatMessageActiveModel {
        id: ActiveValue::Set(message_id),
        room_id: ActiveValue::Set(room_id),
        user_id: ActiveValue::Set(auth.id),
        content: ActiveValue::Set(format!("[audio]({})", audio_url)),
        created_at: ActiveValue::Set(Utc::now()),
        updated_at: ActiveValue::Set(Utc::now()),
    };

    // Debug: Log the message data before inserting
    log::info!("Attempting to insert voice message: room_id={}, user_id={}, content={}", 
        room_id, auth.id, format!("[audio]({})", audio_url));
    
    // Simple database insert without complex validation
    match ChatMessage::insert(message).exec(db.get_ref()).await {
        Ok(result) => {
            log::info!("Successfully inserted voice message with result: {:?}", result);
            
            // Broadcast voice message to all users in the room via WebSocket
            let message_response = WsResponse {
                message_type: "message".to_string(),
                data: serde_json::json!({
                    "id": message_id,
                    "content": format!("[audio]({})", audio_url),
                    "user_id": auth.id,
                    "user_name": auth.name,
                    "user_profile_image": auth.profile_image,
                    "room_id": room_id,
                    "timestamp": Utc::now().timestamp()
                }),
                timestamp: Utc::now().timestamp(),
                message_id: Some(message_id.to_string()),
            };

            // Broadcast voice message to all users in the room via WebSocket
            let chat_server = CHAT_SERVER.lock().unwrap();
            chat_server.broadcast_to_room(&room_id.to_string(), &message_response, None);

            Ok(HttpResponse::Created().json(serde_json::json!({
                "success": true,
                "message_id": message_id,
                "audio_url": audio_url,
                "filename": unique_filename
            })))
        }
        Err(e) => {
            log::error!("Failed to save voice message to database: {:?}", e);
            log::error!("Error details: {}", e);
            // Clean up file if database insert fails
            let _ = std::fs::remove_file(file_path);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to save voice message: {}", e)
            })))
        }
    }
} 

#[get("/api/chat/voice/{filename:.*}")]
pub async fn get_voice_message(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    filename: web::Path<String>,
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
    let user_id = match claims.backend_user_id {
        Some(id) => match Uuid::parse_str(&id) {
            Ok(id) => id,
            Err(_) => {
                return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized: Invalid user ID format"
                })));
            }
        },
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized: Missing user ID in token"
            })));
        }
    };

    // Simple file existence check - remove complex database checks for now
    log::info!("Requesting voice file: {}", filename);

    // Construct file path
    let file_path = Path::new("./uploads/voice_messages").join(&*filename);
    
    // Check if file exists
    if !file_path.exists() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Audio file not found"
        })));
    }

    // Read file content
    let audio_data = fs::read(&file_path).map_err(|e| {
        log::error!("Failed to read audio file: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to read audio file")
    })?;

    // Determine content type based on file extension
    let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("m4a") => "audio/mp4",
        Some("aac") => "audio/aac",
        Some("flac") => "audio/flac",
        _ => "audio/webm", // Default
    };

    // Return audio file with appropriate headers
    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .append_header(("Content-Disposition", format!("inline; filename=\"{}\"", filename)))
        .body(audio_data))
}

