use actix_web::{web, post, HttpResponse, Responder, HttpRequest, http::header};
use actix_multipart::Multipart;
use actix_files::NamedFile;
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait};
use futures::{StreamExt, TryStreamExt};
use std::io::Write;
use std::path::Path;
use std::fs::{self, File};
use std::env;
use uuid::Uuid;
use log::{debug, error, info, warn};
use jsonwebtoken::{decode, DecodingKey, Validation, errors::ErrorKind};

use crate::auth::Claims;
use crate::api::auth::is_token_blacklisted;
use crate::models::entities::{User, UserActiveModel};

// Constants for file upload
const UPLOAD_DIR: &str = "uploads/profile_pictures";
const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
const ALLOWED_TYPES: [&str; 3] = ["image/jpeg", "image/png", "image/gif"];

#[post("/api/user/profile/upload")]
pub async fn upload_profile_picture(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    mut payload: Multipart,
) -> impl Responder {
    // Try to extract the token from the secure `HttpOnly` cookie first.
    let mut token_str: Option<String> = None;

    // 1. Try to extract the token from the secure `HttpOnly` cookie first.
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found 'auth_token' cookie for profile upload endpoint");
        token_str = Some(cookie.value().to_string());
    }

    // 2. If no cookie is found, fall back to checking the Authorization header.
    if token_str.is_none() {
        debug!("No 'auth_token' cookie, checking Authorization header for profile upload endpoint");
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
            warn!("Authentication token not found for profile upload endpoint");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Authentication token is missing"
            }));
        }
    };

    // Check if token is blacklisted
    if is_token_blacklisted(&token) {
        warn!("Token is blacklisted");
        return HttpResponse::Unauthorized().json(
            serde_json::json!({
                "error": "Unauthorized",
                "message": "Token has been invalidated"
            })
        );
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
                    return HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    );
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    return HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    );
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    return HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid token"
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
            warn!("Missing backend_user_id in token");
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Missing user ID in token"
                })
            );
        }
    };

    let user_id = match Uuid::parse_str(&backend_user_id) {
        Ok(id) => id,
        Err(e) => {
            warn!("Invalid backend_user_id format in token: {:?}", e);
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid user ID format in token"
                })
            );
        }
    };
    
    debug!("Uploading profile picture for user ID: {}", user_id);
    
    // Ensure upload directory exists
    if let Err(e) = fs::create_dir_all(UPLOAD_DIR) {
        error!("Failed to create upload directory: {:?}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Server error",
            "message": "Failed to create upload directory"
        }));
    }
    
    // Process the multipart form
    while let Ok(Some(mut field)) = payload.try_next().await {
        // Check if this is the file field
        if field.name() == Some("file") {
            // Get content type and check if it's allowed
            let content_disposition = field.content_disposition();
            let content_type = field.content_type();
            
            // Get the content type string
            let content_type_str = content_type.map(|ct| ct.to_string()).unwrap_or_else(|| "application/octet-stream".to_string());
            
            if !ALLOWED_TYPES.contains(&content_type_str.as_str()) {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid file type",
                    "message": "Only JPEG, PNG, and GIF images are allowed"
                }));
            }
            
            // Generate a unique filename
            let file_ext = match content_type_str.as_str() {
                "image/jpeg" => "jpg",
                "image/png" => "png",
                "image/gif" => "gif",
                "image/webp" => "webp",
                "image/svg+xml" => "svg",
                _ => "bin", // Should not happen due to check above
            };
            
            // Use UUID for filename to prevent path traversal attacks
            let uuid = Uuid::new_v4();
            let filename = format!("{}.{}", uuid, file_ext);
            
            // Ensure the filepath is properly constructed and sanitized
            let filepath = format!("{}/{}", UPLOAD_DIR, filename);
            
            // Create the file
            let mut file = match File::create(&filepath) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to create file: {:?}", e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Server error",
                        "message": "Failed to create file"
                    }));
                }
            };
            
            // Write the file data
            let mut size: usize = 0;
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(data) => data,
                    Err(e) => {
                        error!("Error reading chunk: {:?}", e);
                        // Clean up the partial file
                        if let Err(cleanup_err) = fs::remove_file(&filepath) {
                            error!("Failed to clean up partial file after error: {:?}", cleanup_err);
                        }
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Server error",
                            "message": "Failed to read uploaded file"
                        }));
                    }
                };
                
                // Check file size limit
                size += data.len();
                if size > MAX_FILE_SIZE {
                    // Delete the partial file
                    if let Err(cleanup_err) = fs::remove_file(&filepath) {
                        error!("Failed to clean up oversized file: {:?}", cleanup_err);
                    } else {
                        debug!("Successfully cleaned up oversized file: {}", filepath);
                    }
                    
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "File too large",
                        "message": "Maximum file size is 5MB"
                    }));
                }
                
                // Write the chunk to the file
                if let Err(e) = file.write_all(&data) {
                    error!("Error writing to file: {:?}", e);
                    
                    // Close the file handle before attempting to remove it
                    drop(file);
                    
                    // Clean up the partial file
                    if let Err(cleanup_err) = fs::remove_file(&filepath) {
                        error!("Failed to clean up partial file after write error: {:?}", cleanup_err);
                    } else {
                        debug!("Successfully cleaned up partial file after write error: {}", filepath);
                    }
                    
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Server error",
                        "message": "Failed to write to file"
                    }));
                }
            }
            
            // Update the user's profile_image field
            let user_result = User::find_by_id(user_id)
                .one(db.get_ref())
                .await;
            
            match user_result {
                Ok(Some(user)) => {
                    // Create the URL path for the profile image
                    // Get the backend URL from the environment variable or use a default
                    // Try backend-specific env var first, then fallback to frontend env var
                    let backend_url = env::var("API_URL").or_else(|_| env::var("BACKEND_URL")).unwrap_or_else(|_| {
                        // Fallback to frontend env var if backend-specific ones are not set
                        env::var("NEXT_PUBLIC_API_URL").unwrap_or_else(|_| {
                            warn!("No API URL environment variables set (API_URL, BACKEND_URL, NEXT_PUBLIC_API_URL), using default value");
                            "http://localhost:8080".to_string()
                        })
                    });
                    
                    // Construct the full URL including the backend server
                    let profile_image_url = format!("{}/api/user/profile/image/{}", backend_url, filename);
                    
                    // Update the user model
                    let mut user_active: UserActiveModel = user.into();
                    user_active.profile_image = Set(Some(profile_image_url.clone()));
                    
                    match user_active.update(db.get_ref()).await {
                        Ok(_) => {
                            info!("Updated profile picture for user {}", user_id);
                            return HttpResponse::Ok().json(serde_json::json!({
                                "success": true,
                                "profile_image": profile_image_url
                            }));
                        }
                        Err(e) => {
                            error!("Failed to update user profile: {:?}", e);
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Database error",
                                "message": "Failed to update user profile"
                            }));
                        }
                    }
                }
                Ok(None) => {
                    error!("User not found: {}", user_id);
                    return HttpResponse::NotFound().json(serde_json::json!({
                        "error": "User not found"
                    }));
                }
                Err(e) => {
                    error!("Database error when fetching user {}: {:?}", user_id, e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Database error",
                        "message": "Failed to retrieve user information"
                    }));
                }
            }
        }
    }
    
    // If we get here, no file was uploaded
    HttpResponse::BadRequest().json(serde_json::json!({
        "error": "No file uploaded",
        "message": "Please provide a file"
    }))
}

#[actix_web::get("/api/user/profile/image/{filename}")]
pub async fn get_profile_image(
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let filename = path.into_inner();
    let path = format!("{}/{}", UPLOAD_DIR, filename);
    
    // Check if file exists
    if !Path::new(&path).exists() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "File not found"
        }));
    }
    
    // Serve the file
    match NamedFile::open(path) {
        Ok(file) => file.into_response(&req),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to open file"
        })),
    }
}