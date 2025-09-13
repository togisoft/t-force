use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveValue, ActiveModelTrait, QueryFilter, ColumnTrait, Set};
use serde::{Deserialize, Serialize};
use log::{info, error, debug, warn};
use chrono::{Utc, Duration};
use rand::{rng, Rng};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use lettre::{
    Message, 
    SmtpTransport, 
    Transport, 
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
};
use rand::distr::Alphanumeric;
use std::fs;
use std::path::Path;
use crate::models::{User, UserActiveModel, entities::user::Column};

// Request to initiate password reset
#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

// Response for password reset request
#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

// Request to reset password with token
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}

// Response for password reset
#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

// Generate a random token
fn generate_reset_token() -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

// Send password reset email
async fn send_password_reset_email(email: &str, token: &str) -> Result<(), String> {
    // Get SMTP settings from environment variables with better error handling
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| {
        warn!("SMTP_HOST not set, using default: smtp.gmail.com");
        "smtp.gmail.com".to_string()
    });
    
    let smtp_port = match std::env::var("SMTP_PORT") {
        Ok(port) => {
            // Validate that the port is a valid number
            match port.parse::<u16>() {
                Ok(_) => port,
                Err(_) => {
                    warn!("SMTP_PORT is not a valid port number, using default: 587");
                    "587".to_string()
                }
            }
        },
        Err(_) => {
            warn!("SMTP_PORT not set, using default: 587");
            "587".to_string()
        }
    };
    
    let smtp_username = match std::env::var("SMTP_USERNAME") {
        Ok(username) => username,
        Err(_) => {
            error!("SMTP_USERNAME environment variable is not set");
            return Err("Email configuration error: SMTP_USERNAME is not set. Please configure your email settings.".to_string());
        }
    };
    
    let smtp_password = match std::env::var("SMTP_PASSWORD") {
        Ok(password) => password,
        Err(_) => {
            error!("SMTP_PASSWORD environment variable is not set");
            return Err("Email configuration error: SMTP_PASSWORD is not set. Please configure your email settings.".to_string());
        }
    };
    
    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| {
        warn!("FRONTEND_URL not set, using default: http://localhost:3000");
        "http://localhost:3000".to_string()
    });
    
    // Create the reset URL
    let reset_url = format!("{}/reset-password?token={}", frontend_url, token);
    
    // Create the email
    let email_message = Message::builder()
        .from(format!("T-Force <{}>", smtp_username).parse().map_err(|e| format!("Invalid from address: {}", e))?)
        .to(email.parse().map_err(|e| format!("Invalid to address: {}", e))?)
        .subject("Password Reset Request")
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(format!("You requested a password reset. Please click the following link to reset your password: {}", reset_url))
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body({
                            // Load the HTML template from file
                            let template_path = Path::new("backend/templates/password_reset.html");
                            let template = match fs::read_to_string(template_path) {
                                Ok(content) => content,
                                Err(e) => {
                                    error!("Failed to read password reset template: {}", e);
                                    // Fallback to a simple HTML template if file can't be read
                                    format!(r#"<html><body><p>Reset your password by clicking <a href="{}">here</a>.</p></body></html>"#, reset_url)
                                }
                            };
                            
                            // Replace the placeholders with the actual reset URL
                            template.replace("{reset_url}", &reset_url)
                        })
                )
        )
        .map_err(|e| format!("Failed to build email: {}", e))?;
    
    // Create SMTP transport with better error handling
    let creds = Credentials::new(smtp_username.clone(), smtp_password);
    
    // Parse SMTP port with better error handling
    let smtp_port_num = match smtp_port.parse::<u16>() {
        Ok(port) => port,
        Err(e) => {
            error!("Invalid SMTP port '{}': {}", smtp_port, e);
            return Err("Email configuration error: Invalid SMTP port. Please check your email settings.".to_string());
        }
    };
    
    // Create SMTP relay with better error handling
    let transport_builder = match SmtpTransport::relay(&smtp_host) {
        Ok(builder) => builder,
        Err(e) => {
            error!("Failed to create SMTP relay for host '{}': {}", smtp_host, e);
            return Err("Email configuration error: Failed to create SMTP transport. Please check your email settings.".to_string());
        }
    };
    
    // Build the mailer
    let mailer = transport_builder
        .credentials(creds)
        .port(smtp_port_num)
        .timeout(Some(std::time::Duration::from_secs(15))) // Add timeout for better reliability
        .build();
    
    // Send the email with detailed error handling
    match mailer.send(&email_message) {
        Ok(_) => {
            info!("Password reset email sent successfully to {}", email);
            Ok(())
        }
        Err(e) => {
            // Log detailed error information for troubleshooting
            error!("Failed to send password reset email to {}: {}", email, e);
            error!("SMTP configuration: host={}, port={}, username={}", smtp_host, smtp_port, smtp_username);
            
            // Return a user-friendly error message
            Err("Failed to send password reset email. Please try again later or contact support if the problem persists.".to_string())
        }
    }
}

// Endpoint to request password reset
#[post("/api/auth/forgot-password")]
pub async fn forgot_password(
    db: web::Data<DatabaseConnection>,
    req: web::Json<ForgotPasswordRequest>,
) -> impl Responder {
    let email = &req.email;
    debug!("Password reset requested for email: {}", email);
    
    // Find user by email
    let user = match User::find()
        .filter(Column::Email.eq(email))
        .one(db.get_ref())
        .await {
            Ok(Some(user)) => user,
            Ok(None) => {
                // Don't reveal that the email doesn't exist for security reasons
                debug!("No user found with email: {}", email);
                return HttpResponse::Ok().json(ForgotPasswordResponse {
                    message: "If your email is registered, you will receive a password reset link.".to_string(),
                });
            }
            Err(e) => {
                error!("Database error when finding user by email: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Failed to process request"})
                );
            }
        };
    
    // Generate a reset token
    let reset_token = generate_reset_token();
    
    // Set token expiration (1 hour from now)
    let expires = Utc::now() + Duration::hours(1);
    
    // Update user with reset token and expiration
    let mut user_active: UserActiveModel = user.into();
    user_active.password_reset_token = Set(Some(reset_token.clone()));
    user_active.password_reset_expires = Set(Some(expires));
    
    match user_active.update(db.get_ref()).await {
        Ok(_) => {
            // Send password reset email
            match send_password_reset_email(email, &reset_token).await {
                Ok(_) => {
                    HttpResponse::Ok().json(ForgotPasswordResponse {
                        message: "If your email is registered, you will receive a password reset link.".to_string(),
                    })
                }
                Err(e) => {
                    error!("Failed to send password reset email: {}", e);
                    HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": "Failed to send password reset email"})
                    )
                }
            }
        }
        Err(e) => {
            error!("Database error when updating user with reset token: {:?}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to process request"})
            )
        }
    }
}

// Endpoint to reset password with token
#[post("/api/auth/reset-password")]
pub async fn reset_password(
    db: web::Data<DatabaseConnection>,
    req: web::Json<ResetPasswordRequest>,
) -> impl Responder {
    let token = &req.token;
    let new_password = &req.password;
    
    debug!("Password reset requested with token");
    
    // Find user by reset token
    let user = match User::find()
        .filter(Column::PasswordResetToken.eq(token))
        .one(db.get_ref())
        .await {
            Ok(Some(user)) => user,
            Ok(None) => {
                warn!("Invalid or expired password reset token");
                return HttpResponse::BadRequest().json(
                    serde_json::json!({"error": "Invalid or expired password reset token"})
                );
            }
            Err(e) => {
                error!("Database error when finding user by reset token: {:?}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": "Failed to process request"})
                );
            }
        };
    
    // Check if token is expired
    if let Some(expires) = user.password_reset_expires {
        if expires < Utc::now() {
            warn!("Password reset token has expired");
            return HttpResponse::BadRequest().json(
                serde_json::json!({
                    "error": "Password reset token has expired",
                    "message": "The password reset link has expired. Please request a new one."
                })
            );
        }
    } else {
        warn!("Password reset token has no expiration");
        return HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Invalid password reset token",
                "message": "The password reset link is invalid. Please request a new one."
            })
        );
    }
    
    // Hash the new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(new_password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            error!("Failed to hash password: {:?}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to process request"})
            );
        }
    };
    
    // Update user with new password and clear reset token
    let mut user_active: UserActiveModel = user.into();
    user_active.password_hash = Set(Some(password_hash));
    user_active.password_reset_token = Set(None);
    user_active.password_reset_expires = Set(None);
    user_active.updated_at = Set(Utc::now());
    
    match user_active.update(db.get_ref()).await {
        Ok(_) => {
            info!("Password reset successful");
            HttpResponse::Ok().json(ResetPasswordResponse {
                message: "Password has been reset successfully".to_string(),
            })
        }
        Err(e) => {
            error!("Database error when updating user password: {:?}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to reset password"})
            )
        }
    }
}

