use actix_web::{get, web, HttpResponse, Responder, http::header, HttpRequest};
use log::{debug, error, warn};
use serde::Serialize;
use jsonwebtoken::{decode, DecodingKey, Validation, errors::ErrorKind};
use uuid::Uuid;

use crate::auth::Claims;
use crate::api::auth::is_token_blacklisted;

#[derive(Debug, Serialize)]
pub struct ValidateSessionResponse {
    pub authenticated: bool,
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

/// Endpoint to validate a session token
/// This endpoint validates the token directly without relying on middleware
#[get("/api/auth/validate")]
pub async fn validate_session(
    req: HttpRequest,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for validate endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for validate endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for validate endpoint");
                return session_invalid();
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(e) => {
                warn!("Invalid Authorization header format: {:?}", e);
                return session_invalid();
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return session_invalid();
        }

        token_str = Some(auth_str.trim_start_matches("Bearer ").trim().to_string());
    }
    
    // Unwrap the token (we know it's Some at this point)
    let token = token_str.unwrap();

    // Check if the token is blacklisted
    if is_token_blacklisted(&token) {
        warn!("Token is blacklisted");
        return session_invalid();
    }

    // Configure validation to explicitly check expiration
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.leeway = 0; // No leeway for expiration time

    // Validate the token
    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(e) => {
            match e.kind() {
                ErrorKind::ExpiredSignature => {
                    warn!("Token expired");
                    return session_invalid();
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    return session_invalid();
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    return session_invalid();
                }
            }
        }
    };

    let claims = token_data.claims;

    // Ensure backend_user_id is present
    let backend_user_id = match claims.backend_user_id {
        Some(id) => id,
        None => {
            warn!("Missing backend_user_id in token");
            return session_invalid();
        }
    };

    let user_id = match Uuid::parse_str(&backend_user_id) {
        Ok(id) => id,
        Err(e) => {
            warn!("Invalid backend_user_id format in token: {:?}", e);
            return session_invalid();
        }
    };

    let role = claims.user_role.unwrap_or_else(|| "user".to_string());
    
    debug!("Session validated for user ID: {}", user_id);
    
    HttpResponse::Ok().json(ValidateSessionResponse {
        authenticated: true,
        user_id: Some(user_id.to_string()),
        email: Some(claims.email),
        role: Some(role),
    })
}

/// Handler for when authentication fails
/// This is not an actual endpoint, but a helper function for the error case
pub fn session_invalid() -> HttpResponse {
    error!("Session validation failed - invalid or missing token");
    
    HttpResponse::Unauthorized().json(ValidateSessionResponse {
        authenticated: false,
        user_id: None,
        email: None,
        role: None,
    })
}