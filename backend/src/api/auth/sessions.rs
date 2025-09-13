use actix_web::{get, delete, web, HttpRequest, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveValue, ActiveModelTrait};
use log::{info, error, debug};
use uuid::Uuid;
use chrono::Utc;

use crate::models::entities::{UserSession, UserSessionModel, UserSessionActiveModel, SessionResponseDto};
use crate::auth::{extract_user_id_from_token, extract_token_from_cookie_or_header};

#[get("/api/auth/sessions")]
pub async fn get_sessions(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> impl Responder {
    // Extract user ID from a token
    let user_id = match extract_user_id_from_token(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "Unauthorized"})
            );
        }
    };

    debug!("Fetching sessions for user: {}", user_id);

    // Get the current session ID from the token
    let token = match extract_token_from_cookie_or_header(&req) {
        Some(token) => token,
        None => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "Unauthorized"})
            );
        }
    };

    // Find all active sessions for the user
    let sessions = match UserSession::find()
        .filter(crate::models::entities::user_session::Column::UserId.eq(user_id))
        .filter(crate::models::entities::user_session::Column::IsActive.eq(true))
        .all(db.get_ref())
        .await {
            Ok(sessions) => sessions,
            Err(e) => {
                error!("Database error when finding sessions: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Database error when finding sessions"})
                );
            }
        };

    // Convert to DTOs and mark the current session
    let mut session_dtos: Vec<SessionResponseDto> = sessions
        .into_iter()
        .map(|session| {
            let mut dto: SessionResponseDto = session.into();
            // Check if this is the current session by comparing the token
            // In a real implementation, you would store a session token or ID in the session record
            // For now, we'll just mark the most recent session as current
            dto.is_current = false; // We'll set this below
            dto
        })
        .collect();

    // Sort sessions by last_active_at (most recent first)
    session_dtos.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));

    // Mark the first session as current (most recent)
    if !session_dtos.is_empty() {
        session_dtos[0].is_current = true;
    }

    HttpResponse::Ok().json(session_dtos)
}

#[delete("/api/auth/sessions/{session_id}")]
pub async fn terminate_session(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let session_id = path.into_inner();
    
    // Extract user ID from token
    let user_id = match extract_user_id_from_token(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "Unauthorized"})
            );
        }
    };

    debug!("Terminating session {} for user: {}", session_id, user_id);

    // Parse session ID
    let session_uuid = match Uuid::parse_str(&session_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Invalid session ID format"})
            );
        }
    };

    // Find the session
    let session = match UserSession::find_by_id(session_uuid)
        .one(db.get_ref())
        .await {
            Ok(Some(session)) => session,
            Ok(None) => {
                return HttpResponse::NotFound().json(
                    serde_json::json!({"error": "Session not found"})
                );
            }
            Err(e) => {
                error!("Database error when finding session: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Database error when finding session"})
                );
            }
        };

    // Verify the session belongs to the user
    if session.user_id != user_id {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"error": "You don't have permission to terminate this session"})
        );
    }

    // Update the session to mark it as inactive
    let mut session_active: UserSessionActiveModel = session.into();
    session_active.is_active = Set(false);
    session_active.updated_at = Set(Utc::now());

    match session_active.update(db.get_ref()).await {
        Ok(_) => {
            info!("Session terminated: {}", session_id);
            HttpResponse::Ok().json(
                serde_json::json!({"message": "Session terminated successfully"})
            )
        }
        Err(e) => {
            error!("Database error when updating session: {:?}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Database error when updating session"})
            )
        }
    }
}

#[delete("/api/auth/sessions")]
pub async fn terminate_all_sessions(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> impl Responder {
    // Extract user ID from token
    let user_id = match extract_user_id_from_token(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "Unauthorized"})
            );
        }
    };

    debug!("Terminating all sessions for user: {}", user_id);

    // Get the current session ID from the token
    let token = match extract_token_from_cookie_or_header(&req) {
        Some(token) => token,
        None => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "Unauthorized"})
            );
        }
    };

    // Find all active sessions for the user
    let sessions = match UserSession::find()
        .filter(crate::models::entities::user_session::Column::UserId.eq(user_id))
        .filter(crate::models::entities::user_session::Column::IsActive.eq(true))
        .all(db.get_ref())
        .await {
            Ok(sessions) => sessions,
            Err(e) => {
                error!("Database error when finding sessions: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Database error when finding sessions"})
                );
            }
        };

    // Sort sessions by last_active_at (most recent first)
    let mut sorted_sessions = sessions;
    sorted_sessions.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));

    // Keep the most recent session (current session) active
    if sorted_sessions.is_empty() {
        return HttpResponse::Ok().json(
            serde_json::json!({"message": "No active sessions to terminate"})
        );
    }

    // Skip the first session (most recent/current) and terminate all others
    let sessions_to_terminate = &sorted_sessions[1..];
    
    for session in sessions_to_terminate {
        let mut session_active: UserSessionActiveModel = session.clone().into();
        session_active.is_active = Set(false);
        session_active.updated_at = Set(Utc::now());
        
        if let Err(e) = session_active.update(db.get_ref()).await {
            error!("Database error when updating session {}: {:?}", session.id, e);
            // Continue with other sessions even if one fails
        }
    }

    info!("Terminated {} sessions for user: {}", sessions_to_terminate.len(), user_id);
    HttpResponse::Ok().json(
        serde_json::json!({
            "message": format!("Terminated {} sessions successfully", sessions_to_terminate.len())
        })
    )
}

// Helper function to create a new session
pub async fn create_session(
    db: &DatabaseConnection,
    user_id: Uuid,
    ip_address: String,
    user_agent: String,
    device_type: String,
    browser: String,
    os: String,
) -> Result<UserSessionModel, String> {
    let session_id = Uuid::new_v4();
    let now = Utc::now();
    
    let session = UserSessionActiveModel {
        id: Set(session_id),
        user_id: Set(user_id),
        ip_address: Set(ip_address),
        user_agent: Set(user_agent),
        device_type: Set(device_type),
        browser: Set(browser),
        os: Set(os),
        last_active_at: Set(now),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    
    match session.insert(db).await {
        Ok(session) => {
            info!("Created new session for user: {}", user_id);
            Ok(session)
        }
        Err(e) => {
            error!("Database error when creating session: {:?}", e);
            Err("Database error when creating session".to_string())
        }
    }
}