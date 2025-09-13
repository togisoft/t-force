use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation, errors::ErrorKind};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use crate::api::auth::is_token_blacklisted;
use std::rc::Rc;
use uuid::Uuid;

// JWT claims structure that matches NextAuth.js token
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Claims {
    pub(crate) id: String,
    pub sub: String,
    pub email: String,
    pub name: String,
    pub backend_user_id: Option<String>,
    pub user_role: Option<String>,
    pub profile_image: Option<String>,
    pub provider: Option<String>,
    pub is_active: Option<bool>,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}

// User info extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub profile_image: Option<String>,

}

// Auth middleware factory
pub struct JwtAuth {
    jwt_secret: Rc<String>,
}

impl JwtAuth {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret: Rc::new(jwt_secret),
        }
    }

    // Static method for validating tokens (for WebSocket and other uses)
    pub fn validate_token(token: &str, jwt_secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut validation = Validation::default();
        validation.validate_exp = true;
        validation.leeway = 0;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }
}

// Extract token from cookie or header (helper function)
pub fn extract_token_from_cookie_or_header(req: &actix_web::HttpRequest) -> Option<String> {
    // Try cookie first
    if let Some(cookie) = req.cookie("auth_token") {
        return Some(cookie.value().to_string());
    }

    // Try Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str.trim_start_matches("Bearer ").trim().to_string());
            }
        }
    }

    None
}

// Middleware factory implementation
impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service,
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

// Middleware service
pub struct JwtAuthMiddleware<S> {
    service: S,
    jwt_secret: Rc<String>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let jwt_secret = self.jwt_secret.clone();
        let service = self.service.clone();
        let path = req.path().to_string();

        // List of public paths that don't require authentication
        let public_paths = vec![
            "/",
            "/api/health",
            "/api/auth/login",
            "/api/auth/register",
            "/api/auth/oauth/google",
            "/api/auth/oauth/github",
            "/api/auth/oauth/callback",
            "/oauth/callback",
            "/api/auth/validate", // Add validate endpoint to public paths
        ];

        // Check if the path is public
        if public_paths.iter().any(|&p| path == p) {
            debug!("Public path: {}, skipping authentication", path);
            return Box::pin(async move {
                service.call(req).await
            });
        }

        Box::pin(async move {
            debug!("Processing JWT authentication for path: {}", path);

            // Extract token
            let token = match extract_token_from_cookie_or_header(&req.request()) {
                Some(t) => t,
                None => {
                    warn!("No authentication found for path: {}", path);
                    return Err(ErrorUnauthorized("Authentication required"));
                }
            };

            // Check if the token is blacklisted
            if is_token_blacklisted(&token) {
                warn!("Token is blacklisted for path: {}", path);
                return Err(ErrorUnauthorized("Token has been invalidated"));
            }

            // Validate the token using static method
            let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
                Ok(claims) => claims,
                Err(e) => {
                    match e.kind() {
                        ErrorKind::ExpiredSignature => {
                            warn!("Token expired for path: {}", path);
                            return Err(ErrorUnauthorized("Token expired"));
                        }
                        ErrorKind::InvalidSignature => {
                            warn!("Invalid token signature for path: {}", path);
                            return Err(ErrorUnauthorized("Invalid token signature"));
                        }
                        _ => {
                            warn!("Invalid token for path: {}: {:?}", path, e);
                            return Err(ErrorUnauthorized("Invalid token"));
                        }
                    }
                }
            };

            // Ensure backend_user_id is present
            let backend_user_id = match claims.backend_user_id {
                Some(id) => id,
                None => {
                    warn!("Missing backend_user_id in token for path: {}", path);
                    return Err(ErrorUnauthorized("Missing backend_user_id in token"));
                }
            };

            let user_id = match Uuid::parse_str(&backend_user_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Invalid backend_user_id format in token for path: {}: {:?}", path, e);
                    return Err(ErrorUnauthorized("Invalid backend_user_id format"));
                }
            };

            // Create AuthUser with all required fields
            let auth_user = AuthUser {
                id: user_id,
                email: claims.email,
                name: claims.name,
                role: claims.user_role.unwrap_or_else(|| "user".to_string()),
                profile_image: claims.profile_image,
            };

            debug!("Authenticated user {} with role {} for path: {}", user_id, auth_user.role, path);
            req.extensions_mut().insert(auth_user);

            service.call(req).await
        })
    }
}