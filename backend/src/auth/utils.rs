use actix_web::{HttpRequest, http::header};
use log::{warn, debug};
use uuid::Uuid;
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::auth::Claims;

/// Extract the token from the cookie, Authorization header, or query parameter
/// First tries to extract from the "auth_token" cookie, then Authorization header, then query parameter
pub fn extract_token_from_cookie_or_header(req: &HttpRequest) -> Option<String> {
    // First try to extract from the cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie");
        return Some(cookie.value().to_string());
    }
    
    debug!("No auth_token cookie found, checking Authorization header");
    
    // Try Authorization header
    if let Some(token) = extract_token_from_header(req) {
        return Some(token);
    }
    
    debug!("No Authorization header found, checking query parameter");
    
    // Fall back to query parameter
    if let Some(token) = req.query_string().split('&')
        .find(|param| param.starts_with("token="))
        .and_then(|param| param.split('=').nth(1))
    {
        debug!("Found token in query parameter");
        return Some(token.to_string());
    }
    
    None
}

/// Extract the token from the Authorization header
pub fn extract_token_from_header(req: &HttpRequest) -> Option<String> {
    // Get the Authorization header
    let auth_header = match req.headers().get(header::AUTHORIZATION) {
        Some(header) => header,
        None => {
            warn!("Missing Authorization header");
            return None;
        }
    };

    // Convert the header to a string
    let auth_str = match auth_header.to_str() {
        Ok(str) => str,
        Err(e) => {
            warn!("Invalid Authorization header format: {:?}", e);
            return None;
        }
    };

    // Check if it's a Bearer token
    if !auth_str.starts_with("Bearer ") {
        warn!("Authorization header does not start with 'Bearer '");
        return None;
    }

    // Extract the token
    let token = auth_str.trim_start_matches("Bearer ").trim().to_string();
    Some(token)
}

/// Extract the user ID from the token
pub fn extract_user_id_from_token(req: &HttpRequest) -> Option<Uuid> {
    // Get the JWT secret from app data
    let jwt_secret = match req.app_data::<actix_web::web::Data<String>>() {
        Some(secret) => secret,
        None => {
            warn!("JWT secret not found in app data");
            return None;
        }
    };

    // Extract the token from cookie or header
    let token = match extract_token_from_cookie_or_header(req) {
        Some(token) => token,
        None => {
            warn!("No authentication token found (neither cookie nor Authorization header)");
            return None;
        }
    };

    // Configure validation
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.leeway = 0; // No leeway for expiration time

    // Decode and validate the token
    let token_data = match decode::<Claims>(
        &token,
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
        Ok(id) => {
            debug!("Extracted user ID from token: {}", id);
            Some(id)
        }
        Err(e) => {
            warn!("Invalid backend_user_id format in token: {:?}", e);
            None
        }
    }
}