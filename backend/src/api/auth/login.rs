use actix_web::{post, web, HttpResponse, Responder, HttpRequest};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};
use log::{info, error, debug, warn};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, errors::ErrorKind};
use chrono::{Utc, Duration};
use uuid::Uuid;
use argon2::{
    password_hash::{
        PasswordHash, PasswordVerifier
    },
    Argon2
};

use crate::models::{User, UserResponseDto, entities::user::Column};
use crate::models::entities::{TwoFactorAuth, two_factor_auth::Column as TwoFactorColumn};
use crate::auth::Claims;
use crate::api::auth::create_session;
use totp_rs::{TOTP, Secret, Algorithm};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyTwoFactorRequest {
    pub temp_token: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user: UserResponseDto,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct LoginTwoFactorResponse {
    pub user: UserResponseDto,
    pub requires_2fa: bool,
    pub temp_token: String,
}

#[post("/api/auth/login")]
pub async fn login(
    db: web::Data<DatabaseConnection>,
    jwt_secret: web::Data<String>,
    login_data: web::Json<LoginRequest>,
    req: HttpRequest,
) -> impl Responder {
    let login_data = login_data.into_inner();
    
    debug!("Login attempt for email: {}", login_data.email);
    
    // Extract client information from request headers
    let ip_address = req.connection_info().realip_remote_addr()
        .unwrap_or("unknown").to_string();
    
    let user_agent = req.headers().get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown").to_string();
    
    // Find user by email
    let user = match User::find()
        .filter(Column::Email.eq(&login_data.email))
        .one(db.get_ref())
        .await {
            Ok(Some(user)) => user,
            Ok(None) => {
                warn!("Login attempt for non-existent user: {}", login_data.email);
                return HttpResponse::Unauthorized().json(
                    serde_json::json!({"error": "Invalid email or password"})
                );
            }
            Err(e) => {
                error!("Database error when finding user by email: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Database error when finding user"})
                );
            }
        };
    
    // Check if user has a password (they might be OAuth-only)
    let password_hash = match &user.password_hash {
        Some(hash) => hash,
        None => {
            warn!("Login attempt for user without password (OAuth-only): {}", login_data.email);
            return HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "This account doesn't support password login"})
            );
        }
    };
    
    // Verify password
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to parse stored password hash: {:?}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Internal server error"})
            );
        }
    };
    
    if Argon2::default().verify_password(login_data.password.as_bytes(), &parsed_hash).is_err() {
        warn!("Invalid password for user: {}", login_data.email);
        return HttpResponse::Unauthorized().json(
            serde_json::json!({"error": "Invalid email or password"})
        );
    }
    
    // Check if 2FA is enabled for the user
    let two_factor = match TwoFactorAuth::find()
        .filter(TwoFactorColumn::UserId.eq(user.id))
        .one(db.get_ref())
        .await {
            Ok(Some(two_factor)) => two_factor,
            Ok(None) => {
                // No 2FA record, proceed with normal login
                debug!("No 2FA record found for user: {}", user.id);
                
                // Parse user agent to extract browser, device, and OS information
                let (browser, device_type, os) = parse_user_agent(&user_agent);
                
                // Create a session record
                match create_session(
                    db.get_ref(),
                    user.id,
                    ip_address.clone(),
                    user_agent.clone(),
                    device_type,
                    browser,
                    os,
                ).await {
                    Ok(_) => {
                        info!("Session created for user: {}", user.id);
                    }
                    Err(e) => {
                        error!("Failed to create session: {}", e);
                        // Continue with login even if session creation fails
                    }
                }
                
                return generate_normal_login_response(user, jwt_secret);
            }
            Err(e) => {
                error!("Database error when finding 2FA record: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Database error when checking 2FA status"})
                );
            }
        };

    if two_factor.enabled {
        // 2FA is enabled, return a response indicating 2FA verification is needed
        debug!("2FA is enabled for user: {}, requiring verification", user.id);
        
        // Generate a temporary token with short expiration for 2FA verification
        let now = Utc::now();
        let exp = (now + Duration::minutes(5)).timestamp() as usize; // 5 minute expiration
        let iat = now.timestamp() as usize;
        
        let claims = Claims {
            id: user.id.to_string(),
            sub: user.email.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            backend_user_id: Some(user.id.to_string()),
            user_role: Some(user.role.clone()),
            exp,
            iat,
            jti: Uuid::new_v4().to_string(),
            ..Default::default()
        };
        
        let temp_token = match encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        ) {
            Ok(token) => token,
            Err(e) => {
                error!("Failed to generate temporary JWT token: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Failed to generate temporary authentication token"})
                );
            }
        };
        
        info!("User requires 2FA verification: {}", user.id);
        HttpResponse::Ok().json(LoginTwoFactorResponse {
            user: user.into(),
            requires_2fa: true,
            temp_token,
        })
    } else {
        // 2FA is not enabled, proceed with normal login
        debug!("2FA is not enabled for user: {}", user.id);
        
        // Parse user agent to extract browser, device, and OS information
        let (browser, device_type, os) = parse_user_agent(&user_agent);
        
        // Create a session record
        match create_session(
            db.get_ref(),
            user.id,
            ip_address,
            user_agent,
            device_type,
            browser,
            os,
        ).await {
            Ok(_) => {
                info!("Session created for user: {}", user.id);
            }
            Err(e) => {
                error!("Failed to create session: {}", e);
                // Continue with login even if session creation fails
            }
        }

        generate_normal_login_response(user, jwt_secret)
    }
}

// Helper function to verify a TOTP code
fn verify_totp(secret: &str, code: &str) -> bool {
    // Create a TOTP object from the secret
    let totp = match create_totp(secret) {
        Some(totp) => totp,
        None => return false,
    };
    
    // Verify the code
    totp.check_current(code).unwrap_or(false)
}

// Helper function to create a TOTP object from a secret
fn create_totp(secret: &str) -> Option<TOTP> {
    // Try to decode the Base32 secret
    let secret_bytes = match Secret::Encoded(secret.to_string()).to_bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to decode Base32 secret: {}", e);
            return None;
        }
    };
    
    // Create the TOTP with the secret bytes
    match TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
    ) {
        Ok(totp) => Some(totp),
        Err(e) => {
            error!("Failed to create TOTP: {}", e);
            None
        }
    }
}

#[post("/api/auth/verify-2fa")]
pub async fn verify_two_factor(
    db: web::Data<DatabaseConnection>,
    jwt_secret: web::Data<String>,
    verify_req: web::Json<VerifyTwoFactorRequest>,
    req: HttpRequest,
) -> impl Responder {
    let verify_req = verify_req.into_inner();
    
    debug!("Verifying 2FA code");
    
    // Validate the temporary token
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.leeway = 0; // No leeway for expiration time
    
    // Decode and validate the token
    let token_data = match decode::<Claims>(
        &verify_req.temp_token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(e) => {
            match e.kind() {
                ErrorKind::ExpiredSignature => {
                    warn!("Temporary token expired");
                    return HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Temporary token expired"
                        })
                    );
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid temporary token signature");
                    return HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid temporary token signature"
                        })
                    );
                }
                _ => {
                    warn!("Invalid temporary token: {:?}", e);
                    return HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid temporary token"
                        })
                    );
                }
            }
        }
    };
    
    let claims = token_data.claims;
    
    // Ensure backend_user_id is present
    let backend_user_id = match claims.backend_user_id {
        Some(id) => id,
        None => {
            warn!("Missing backend_user_id in temporary token");
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Missing user ID in temporary token"
                })
            );
        }
    };
    
    let user_id = match Uuid::parse_str(&backend_user_id) {
        Ok(id) => id,
        Err(e) => {
            warn!("Invalid backend_user_id format in temporary token: {:?}", e);
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid user ID format in temporary token"
                })
            );
        }
    };
    
    debug!("Verifying 2FA code for user ID: {}", user_id);
    
    // Find the user
    let user = match User::find_by_id(user_id).one(db.get_ref()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("User not found for ID: {}", user_id);
            return HttpResponse::NotFound().json(
                serde_json::json!({
                    "error": "Not Found",
                    "message": "User not found"
                })
            );
        }
        Err(e) => {
            error!("Database error when finding user: {:?}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Database error when finding user"
                })
            );
        }
    };
    
    // Get 2FA record
    let two_factor = match TwoFactorAuth::find()
        .filter(TwoFactorColumn::UserId.eq(user_id))
        .one(db.get_ref())
        .await {
            Ok(Some(two_factor)) => two_factor,
            Ok(None) => {
                warn!("2FA record not found for user: {}", user_id);
                return HttpResponse::NotFound().json(
                    serde_json::json!({
                        "error": "Not Found",
                        "message": "Two-factor authentication not set up"
                    })
                );
            }
            Err(e) => {
                error!("Database error when finding 2FA record: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({
                        "error": "Internal Server Error",
                        "message": "Database error when finding 2FA record"
                    })
                );
            }
        };
    
    // Verify the code
    if !verify_totp(&two_factor.secret, &verify_req.code) {
        warn!("Invalid 2FA code for user: {}", user_id);
        return HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Bad Request",
                "message": "Invalid verification code"
            })
        );
    }
    
    // Code is valid, generate a standard JWT token
    info!("2FA verification successful for user: {}", user_id);
    
    // Extract client information from request headers
    let ip_address = req.connection_info().realip_remote_addr()
        .unwrap_or("unknown").to_string();
    
    let user_agent = req.headers().get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown").to_string();
    
    // Parse user agent to extract browser, device, and OS information
    let (browser, device_type, os) = parse_user_agent(&user_agent);
    
    // Create a session record
    match create_session(
        db.get_ref(),
        user_id,
        ip_address,
        user_agent,
        device_type,
        browser,
        os,
    ).await {
        Ok(_) => {
            info!("Session created for user: {}", user_id);
        }
        Err(e) => {
            error!("Failed to create session: {}", e);
            // Continue with login even if session creation fails
        }
    }
    
    generate_normal_login_response(user, jwt_secret)
}

// Helper function to parse User-Agent string
fn parse_user_agent(user_agent: &str) -> (String, String, String) {
    // Default values
    let mut browser = "Unknown".to_string();
    let mut device_type = "Unknown".to_string();
    let mut os = "Unknown".to_string();
    
    // Very basic parsing - in a real app, you'd use a proper user-agent parsing library
    let ua_lower = user_agent.to_lowercase();
    
    // Detect browser
    if ua_lower.contains("firefox") {
        browser = "Firefox".to_string();
    } else if ua_lower.contains("chrome") && !ua_lower.contains("edg") {
        browser = "Chrome".to_string();
    } else if ua_lower.contains("safari") && !ua_lower.contains("chrome") {
        browser = "Safari".to_string();
    } else if ua_lower.contains("edg") {
        browser = "Edge".to_string();
    } else if ua_lower.contains("opera") || ua_lower.contains("opr") {
        browser = "Opera".to_string();
    }
    
    // Detect OS
    if ua_lower.contains("windows") {
        os = "Windows".to_string();
    } else if ua_lower.contains("mac os") {
        os = "macOS".to_string();
    } else if ua_lower.contains("android") {
        os = "Android".to_string();
    } else if ua_lower.contains("ios") || ua_lower.contains("iphone") || ua_lower.contains("ipad") {
        os = "iOS".to_string();
    } else if ua_lower.contains("linux") {
        os = "Linux".to_string();
    }
    
    // Detect device type
    if ua_lower.contains("mobile") || ua_lower.contains("android") || ua_lower.contains("iphone") {
        device_type = "Mobile".to_string();
    } else if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
        device_type = "Tablet".to_string();
    } else {
        device_type = "Desktop".to_string();
    }
    
    (browser, device_type, os)
}

// Helper function to generate a normal login response with a standard JWT token
fn generate_normal_login_response(
    user: crate::models::entities::user::Model,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    // Generate JWT token
    let now = Utc::now();
    let exp = (now + Duration::hours(24)).timestamp() as usize; // 24 hour expiration
    let iat = now.timestamp() as usize;
    
    let claims = Claims {
        id: user.id.to_string(),
        sub: user.email.clone(),
        email: user.email.clone(),
        name: user.name.clone(),
        backend_user_id: Some(user.id.to_string()),
        user_role: Some(user.role.clone()),
        exp,
        iat,
        jti: Uuid::new_v4().to_string(),
        ..Default::default()
    };
    
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ) {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to generate JWT token: {:?}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to generate authentication token"})
            );
        }
    };
    
    info!("User logged in successfully: {}", user.id);
    
    // Set secure cookie with the JWT token
    // For localhost development, we use SameSite=Lax and Secure=false
    // In production, these should be SameSite=Strict and Secure=true
    let is_production = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()) == "production";
    
    // Convert chrono Duration to seconds for cookie max_age
    let max_age_seconds = Duration::hours(24).num_seconds() as i64;
    
    let cookie = actix_web::cookie::Cookie::build("auth_token", token.clone())
        .path("/")
        .http_only(true)
        .same_site(actix_web::cookie::SameSite::Lax)
        .secure(is_production)
        .max_age(actix_web::cookie::time::Duration::seconds(max_age_seconds))
        .finish();
    
    // Return both cookie and JSON response for backward compatibility
    HttpResponse::Ok()
        .cookie(cookie)
        .json(LoginResponse {
            user: user.into(),
            token,
        })
}