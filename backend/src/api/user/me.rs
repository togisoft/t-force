use actix_web::{get, web, HttpResponse, Responder, http::header, HttpRequest};
use sea_orm::{DatabaseConnection, EntityTrait};
use log::{debug, error, warn};
use jsonwebtoken::{decode, DecodingKey, Validation, errors::ErrorKind};
use uuid::Uuid;

use crate::auth::Claims;
use crate::api::auth::is_token_blacklisted;
use crate::models::{User, UserResponseDto};

#[get("/api/me")]
pub async fn get_current_user(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> impl Responder {
    let mut token_str: Option<String> = None;

    // 1. Try to extract the token from the secure `HttpOnly` cookie first.
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found 'auth_token' cookie for /api/me");
        token_str = Some(cookie.value().to_string());
    }

    // 2. If no cookie is found, fall back to checking the Authorization header.
    if token_str.is_none() {
        debug!("No 'auth_token' cookie, checking Authorization header for /api/me");
        if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    token_str = Some(auth_str[7..].to_string());
                }
            }
        }
    }

    // 3. If no token was found in either place, return an Unauthorized error.
    let token = match token_str {
        Some(t) => t,
        None => {
            warn!("Authentication token not found for /api/me");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Authentication token is missing"
            }));
        }
    };

    // Check if token is blacklisted
    if is_token_blacklisted(&token) {
        warn!("Blacklisted token used for /api/me");
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Token has been invalidated"
        }));
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
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Token has expired"
                }));
            }
            _ => {
                warn!("Invalid token for /api/me: {:?}", e);
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid token"
                }));
            }
        },
    };

    let claims = token_data.claims;
    let user_id_str = match claims.backend_user_id {
        Some(id) => id,
        None => {
            warn!("Token missing backend_user_id for /api/me");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Token missing user ID"
            }));
        }
    };

    let user_id = match Uuid::parse_str(&user_id_str) {
        Ok(id) => id,
        Err(e) => {
            warn!("Invalid user ID format in token: {} - Error: {:?}", user_id_str, e);
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Invalid user ID format"
            }));
        }
    };

    // Fetch the user from the database
    match User::find_by_id(user_id).one(db.as_ref()).await {
        Ok(Some(user)) => {
            let user_dto: UserResponseDto = user.into();
            HttpResponse::Ok().json(user_dto)
        }
        Ok(None) => {
            error!("User with ID {} from valid JWT not found in DB. This could mean the user was deleted or the database was reset.", user_id);
            
            // Return a specific error that indicates the user doesn't exist
            // This will help the frontend handle this case appropriately
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "UserNotFound",
                "message": "User account no longer exists. Please log in again.",
                "details": "The user account associated with this token has been deleted or is no longer available."
            }))
        }
        Err(e) => {
            error!("Database error fetching user {}: {:?}", user_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "DatabaseError",
                "message": "Database error occurred while fetching user information"
            }))
        }
    }
}