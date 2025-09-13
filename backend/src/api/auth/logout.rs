use actix_web::{post, get, web, HttpResponse, Responder, HttpRequest, cookie::Cookie};
use serde::{Deserialize, Serialize};
use log::{info, debug, error, warn};
use std::sync::Mutex;
use std::collections::HashSet;
use lazy_static::lazy_static;
use chrono::{DateTime, Utc};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use uuid::Uuid;

use crate::auth::{AuthUser, Claims};
use crate::models::entities::{UserSession, UserSessionActiveModel};
use jsonwebtoken::{decode, DecodingKey, Validation};

// In a production environment, this should be replaced with a Redis cache or similar
lazy_static! {
    static ref TOKEN_BLACKLIST: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

#[post("/api/auth/logout")]
pub async fn logout(
    db: web::Data<DatabaseConnection>,
    auth_user: web::ReqData<AuthUser>,
    logout_data: web::Json<LogoutRequest>,
) -> impl Responder {
    let logout_data = logout_data.into_inner();
    let user_id = auth_user.id;
    
    debug!("Logging out user: {}", user_id);
    
    // Add token to blacklist
    let mut blacklist = TOKEN_BLACKLIST.lock().unwrap();
    blacklist.insert(logout_data.token);
    
    // Find the most recent active session for the user
    let sessions = UserSession::find()
        .filter(crate::models::entities::user_session::Column::UserId.eq(user_id))
        .filter(crate::models::entities::user_session::Column::IsActive.eq(true))
        .all(db.get_ref())
        .await.unwrap_or_else(|e| {
        error!("Database error when finding sessions: {:?}", e);
        // Continue with logout even if session termination fails
        vec![]
    });
    
    // Sort sessions by last_active_at (most recent first)
    let mut sorted_sessions = sessions;
    sorted_sessions.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));
    
    // Terminate the most recent session (current session)
    if !sorted_sessions.is_empty() {
        let current_session = &sorted_sessions[0];
        let mut session_active: UserSessionActiveModel = current_session.clone().into();
        session_active.is_active = Set(false);
        session_active.updated_at = Set(Utc::now());
        
        match session_active.update(db.get_ref()).await {
            Ok(_) => {
                info!("Session terminated for user: {}", user_id);
            }
            Err(e) => {
                error!("Database error when terminating session: {:?}", e);
                // Continue with logout even if session termination fails
            }
        }
    }
    
    info!("User logged out successfully: {}", user_id);
    
    // Create cookies with the same names but with Max-Age=0 to remove them
    let remove_auth_token = actix_web::cookie::Cookie::build("auth_token", "")
        .path("/")
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();
    
    let remove_auth_user = actix_web::cookie::Cookie::build("auth_user", "")
        .path("/")
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();
    
    HttpResponse::Ok()
        .cookie(remove_auth_token)
        .cookie(remove_auth_user)
        .json(LogoutResponse {
            message: "Logged out successfully".to_string(),
        })
}

#[get("/api/auth/logout")]
pub async fn logout_get(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    debug!("GET request to logout endpoint");
    
    // Try to extract the token from the cookie
    let token = match req.cookie("auth_token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            warn!("No auth_token cookie found for logout");
            // Even if we don't have a token, we should still clear cookies
            return create_logout_response();
        }
    };
    
    // Add token to blacklist
    {
        let mut blacklist = TOKEN_BLACKLIST.lock().unwrap();
        blacklist.insert(token.clone());
    }
    
    // Try to extract user ID from token
    let user_id = match extract_user_id_from_token(&token, jwt_secret.as_ref()) {
        Some(id) => id,
        None => {
            warn!("Could not extract user ID from token");
            // Even if we can't extract the user ID, we should still clear cookies
            return create_logout_response();
        }
    };
    
    debug!("Logging out user: {}", user_id);
    
    // Find the most recent active session for the user
    let sessions = UserSession::find()
        .filter(crate::models::entities::user_session::Column::UserId.eq(user_id))
        .filter(crate::models::entities::user_session::Column::IsActive.eq(true))
        .all(db.get_ref())
        .await.unwrap_or_else(|e| {
        error!("Database error when finding sessions: {:?}", e);
        // Continue with logout even if session termination fails
        vec![]
    });
    
    // Sort sessions by last_active_at (most recent first)
    let mut sorted_sessions = sessions;
    sorted_sessions.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));
    
    // Terminate the most recent session (current session)
    if !sorted_sessions.is_empty() {
        let current_session = &sorted_sessions[0];
        let mut session_active: UserSessionActiveModel = current_session.clone().into();
        session_active.is_active = Set(false);
        session_active.updated_at = Set(Utc::now());
        
        match session_active.update(db.get_ref()).await {
            Ok(_) => {
                info!("Session terminated for user: {}", user_id);
            }
            Err(e) => {
                error!("Database error when terminating session: {:?}", e);
                // Continue with logout even if session termination fails
            }
        }
    }
    
    info!("User logged out successfully: {}", user_id);
    
    create_logout_response()
}

// Helper function to create a logout response with cleared cookies
fn create_logout_response() -> HttpResponse {
    // Create cookies with the same names but with Max-Age=0 to remove them
    let remove_auth_token = Cookie::build("auth_token", "")
        .path("/")
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();
    
    let remove_auth_user = Cookie::build("auth_user", "")
        .path("/")
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();
    
    HttpResponse::Ok()
        .cookie(remove_auth_token)
        .cookie(remove_auth_user)
        .json(LogoutResponse {
            message: "Logged out successfully".to_string(),
        })
}

// Helper function to extract user ID from token
fn extract_user_id_from_token(token: &str, jwt_secret: &str) -> Option<Uuid> {
    // Configure validation
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.leeway = 0; // No leeway for expiration time
    
    // Decode and validate the token
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(e) => {
            warn!("Invalid token: {:?}", e);
            return None;
        }
    };
    
    let claims = token_data.claims;
    
    // Ensure backend_user_id is present
    let backend_user_id = match claims.backend_user_id {
        Some(id) => id,
        None => {
            warn!("Missing backend_user_id in token");
            return None;
        }
    };
    
    // Parse the user ID
    match Uuid::parse_str(&backend_user_id) {
        Ok(id) => Some(id),
        Err(e) => {
            warn!("Invalid backend_user_id format in token: {:?}", e);
            None
        }
    }
}

// Helper function to check if a token is blacklisted
pub fn is_token_blacklisted(token: &str) -> bool {
    let blacklist = TOKEN_BLACKLIST.lock().unwrap();
    blacklist.contains(token)
}

// In a real implementation, you would also want to periodically clean up the blacklist
// to remove expired tokens. This could be done with a background task.
#[derive(Debug, Serialize, Deserialize)]
pub struct BlacklistedToken {
    pub token: String,
    pub expiry: DateTime<Utc>,
}