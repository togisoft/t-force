use actix_web::{web, post, HttpResponse, Responder, HttpRequest};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::io::Write;
use std::path::Path;
use std::fs::{self, File};
use std::env;
use uuid::Uuid;
use log::{debug, error, info, warn};
use actix_web::http::header::{HeaderName, HeaderValue};

use crate::auth::{JwtAuth, extract_token_from_cookie_or_header};

// Constants for file upload
const UPLOAD_DIR: &str = "uploads/chat_images";
const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
const ALLOWED_TYPES: [&str; 3] = ["image/jpeg", "image/png", "image/gif"];

// Video upload constants
const VIDEO_UPLOAD_DIR: &str = "uploads/chat_videos";
const MAX_VIDEO_SIZE: usize = 50 * 1024 * 1024; // 50MB
const ALLOWED_VIDEO_TYPES: [&str; 3] = ["video/mp4", "video/webm", "video/ogg"];
#[post("/api/chat/upload")]
pub async fn upload_chat_image(
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    mut payload: Multipart,
) -> impl Responder {
    // Extract token from request
    let token = match extract_token_from_cookie_or_header(&req) {
        Some(token) => token,
        None => {
            warn!("Missing authentication token for chat image upload");
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Missing authentication token"
                })
            );
        }
    };

    // Validate token and get user
    let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
        Ok(claims) => claims,
        Err(e) => {
            warn!("Invalid token for chat image upload: {:?}", e);
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid authentication token"
                })
            );
        }
    };

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
    
    debug!("Uploading chat image for user ID: {}", user_id);
    
    // Ensure the upload directory exists
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
            let _content_disposition = field.content_disposition();
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
            
            // Verify the file was created and has content
            if size == 0 {
                // Clean up empty file
                if let Err(cleanup_err) = fs::remove_file(&filepath) {
                    error!("Failed to clean up empty file: {:?}", cleanup_err);
                }
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Empty file",
                    "message": "Uploaded file is empty"
                }));
            }
            
            // Create the URL path for the image
            // Get the backend URL from the environment variable or use a default
            let backend_url = env::var("API_URL")
                .or_else(|_| env::var("BACKEND_URL"))
                .unwrap_or_else(|_| {
                    // Fallback to frontend env var if backend-specific ones are not set
                    env::var("NEXT_PUBLIC_API_URL").unwrap_or_else(|_| {
                        warn!("No API URL environment variables set (API_URL, BACKEND_URL, NEXT_PUBLIC_API_URL), using default value");
                        "http://localhost:8080".to_string()
                    })
                });
            
            // Construct the full URL including the backend server
            let image_url = format!("{}/api/chat/image/{}", backend_url, filename);
            
            info!("Successfully uploaded chat image: {} (size: {} bytes)", image_url, size);
            return HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "image_url": image_url,
                "filename": filename,
                "size": size
            }));
        }
    }
    
    // If we get here, no file was uploaded
    HttpResponse::BadRequest().json(serde_json::json!({
        "error": "No file uploaded",
        "message": "Please provide a file"
    }))
}

// Video upload endpoint
#[post("/api/chat/upload-video")]
pub async fn upload_chat_video(
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    mut payload: Multipart,
) -> impl Responder {
    // Authentication
    let token = match extract_token_from_cookie_or_header(&req) {
        Some(token) => token,
        None => {
            warn!("Missing authentication token for chat video upload");
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Missing authentication token"
                })
            );
        }
    };

    let claims = match JwtAuth::validate_token(&token, &jwt_secret) {
        Ok(claims) => claims,
        Err(e) => {
            warn!("Invalid token for chat video upload: {:?}", e);
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid authentication token"
                })
            );
        }
    };

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

    debug!("Uploading chat video for user ID: {}", user_id);

    // Create upload directory
    if let Err(e) = fs::create_dir_all(VIDEO_UPLOAD_DIR) {
        error!("Failed to create video upload directory: {:?}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Server error",
            "message": "Failed to create upload directory"
        }));
    }

    while let Ok(Some(mut field)) = payload.try_next().await {
        if field.name() == Some("file") {
            let content_type = field.content_type();
            let content_type_str = content_type
                .map(|ct| ct.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());

            if !ALLOWED_VIDEO_TYPES.contains(&content_type_str.as_str()) {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid file type",
                    "message": "Only MP4, WebM, and Ogg videos are allowed"
                }));
            }

            let file_ext = match content_type_str.as_str() {
                "video/mp4" => "mp4",
                "video/webm" => "webm",
                "video/ogg" => "ogv",
                _ => "bin",
            };

            let uuid = Uuid::new_v4();
            let filename = format!("{}.{}", uuid, file_ext);
            let filepath = format!("{}/{}", VIDEO_UPLOAD_DIR, filename);

            let mut file = match File::create(&filepath) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to create video file: {:?}", e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Server error",
                        "message": "Failed to create file"
                    }));
                }
            };

            let mut size: usize = 0;
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(data) => data,
                    Err(e) => {
                        error!("Error reading video chunk: {:?}", e);
                        if let Err(cleanup_err) = fs::remove_file(&filepath) {
                            error!("Failed to clean up partial video file after error: {:?}", cleanup_err);
                        }
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Server error",
                            "message": "Failed to read uploaded file"
                        }));
                    }
                };

                size += data.len();
                if size > MAX_VIDEO_SIZE {
                    if let Err(cleanup_err) = fs::remove_file(&filepath) {
                        error!("Failed to clean up oversized video file: {:?}", cleanup_err);
                    } else {
                        debug!("Successfully cleaned up oversized video file: {}", filepath);
                    }

                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "File too large",
                        "message": "Maximum video file size is 50MB"
                    }));
                }

                if let Err(e) = file.write_all(&data) {
                    error!("Error writing to video file: {:?}", e);
                    drop(file);
                    if let Err(cleanup_err) = fs::remove_file(&filepath) {
                        error!("Failed to clean up partial video file after write error: {:?}", cleanup_err);
                    } else {
                        debug!("Successfully cleaned up partial video file after write error: {}", filepath);
                    }

                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Server error",
                        "message": "Failed to write to file"
                    }));
                }
            }

            if size == 0 {
                if let Err(cleanup_err) = fs::remove_file(&filepath) {
                    error!("Failed to clean up empty video file: {:?}", cleanup_err);
                }
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Empty file",
                    "message": "Uploaded file is empty"
                }));
            }

            let backend_url = env::var("API_URL")
                .or_else(|_| env::var("BACKEND_URL"))
                .unwrap_or_else(|_| {
                    env::var("NEXT_PUBLIC_API_URL").unwrap_or_else(|_| {
                        warn!("No API URL environment variables set (API_URL, BACKEND_URL, NEXT_PUBLIC_API_URL), using default value");
                        "http://localhost:8080".to_string()
                    })
                });

            let video_url = format!("{}/api/chat/video/{}", backend_url, filename);

            info!("Successfully uploaded chat video: {} (size: {} bytes)", video_url, size);
            return HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "video_url": video_url,
                "filename": filename,
                "size": size
            }));
        }
    }

    HttpResponse::BadRequest().json(serde_json::json!({
        "error": "No file uploaded",
        "message": "Please provide a file"
    }))
}

// Endpoint to serve video files
#[actix_web::get("/api/chat/video/{filename}")]
pub async fn get_chat_video(
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let filename = path.into_inner();

    let sanitized_filename = Path::new(&filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    if sanitized_filename.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid filename"
        }));
    }

    let filepath = format!("{}/{}", VIDEO_UPLOAD_DIR, sanitized_filename);

    if !Path::new(&filepath).exists() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Video not found"
        }));
    }

    let metadata = match fs::metadata(&filepath) {
        Ok(meta) => meta,
        Err(e) => {
            error!("Failed to get video metadata: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve video metadata"
            }));
        }
    };

    if metadata.len() == 0 {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Video file is empty"
        }));
    }

    let content_type = match Path::new(&sanitized_filename).extension().and_then(|ext| ext.to_str()) {
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("ogv") | Some("ogg") => "video/ogg",
        _ => "application/octet-stream",
    };

    match actix_files::NamedFile::open(filepath) {
        Ok(file) => {
            let mut response = file.into_response(&req);

            if let (Ok(cache_name), Ok(cache_value)) = (
                "Cache-Control".parse::<HeaderName>(),
                "public, max-age=31536000".parse::<HeaderValue>()
            ) {
                response.headers_mut().insert(cache_name, cache_value);
            }

            if let (Ok(content_name), Ok(content_value)) = (
                "Content-Type".parse::<HeaderName>(),
                content_type.parse::<HeaderValue>()
            ) {
                response.headers_mut().insert(content_name, content_value);
            }

            response
        }
        Err(e) => {
            error!("Failed to open video file: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve video"
            }))
        }
    }
}

#[actix_web::get("/api/chat/image/{filename}")]
pub async fn get_chat_image(
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let filename = path.into_inner();
    
    // Sanitize filename to prevent directory traversal
    let sanitized_filename = Path::new(&filename).file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    
    if sanitized_filename.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid filename"
        }));
    }
    
    let filepath = format!("{}/{}", UPLOAD_DIR, sanitized_filename);
    
    // Check if file exists
    if !Path::new(&filepath).exists() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Image not found"
        }));
    }
    
    // Get file metadata
    let metadata = match fs::metadata(&filepath) {
        Ok(meta) => meta,
        Err(e) => {
            error!("Failed to get file metadata: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve image metadata"
            }));
        }
    };
    
    // Check if file is empty
    if metadata.len() == 0 {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Image file is empty"
        }));
    }
    
    // Determine content type based on file extension
    let content_type = match Path::new(&sanitized_filename).extension().and_then(|ext| ext.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    };
    
    // Serve the file with proper headers
    match actix_files::NamedFile::open(filepath) {
        Ok(file) => {
            // Create response with proper headers
            let mut response = file.into_response(&req);
            
            // Add cache headers for better performance
            if let (Ok(cache_name), Ok(cache_value)) = (
                "Cache-Control".parse::<HeaderName>(),
                "public, max-age=31536000".parse::<HeaderValue>()
            ) {
                response.headers_mut().insert(cache_name, cache_value);
            }
            
            // Set content type header
            if let (Ok(content_name), Ok(content_value)) = (
                "Content-Type".parse::<HeaderName>(),
                content_type.parse::<HeaderValue>()
            ) {
                response.headers_mut().insert(content_name, content_value);
            }
            
            response
        }
        Err(e) => {
            error!("Failed to open image file: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve image"
            }))
        }
    }
}