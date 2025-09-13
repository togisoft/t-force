use actix_web::{get, delete, put, web, HttpResponse, Responder, HttpRequest, HttpMessage, http::header};
use sea_orm::{DatabaseConnection, EntityTrait,  Set, ActiveModelTrait};
use serde::{Deserialize, Serialize};
use log::{info, error, debug, warn};
use uuid::Uuid;
use jsonwebtoken::{decode, DecodingKey, Validation, errors::ErrorKind};

use crate::auth::{AuthUser, Claims};
use crate::api::auth::is_token_blacklisted;
use crate::models::{User, UserResponseDto};
use crate::models::entities::user::ActiveModel as UserActiveModel;

// DTO for changing user role
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeRoleRequest {
    pub role: String,
}

// DTO for toggling user active status
#[derive(Debug, Serialize, Deserialize)]
pub struct ToggleActiveRequest {
    pub is_active: bool,
}

// Get all users
#[get("/api/admin/users")]
pub async fn get_all_users(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for /api/admin/users endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for /api/admin/users endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for /api/admin/users endpoint");
                return HttpResponse::Unauthorized().json(
                    serde_json::json!({
                        "error": "Unauthorized",
                        "message": "Authentication required"
                    })
                );
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(e) => {
                warn!("Invalid Authorization header format: {:?}", e);
                return HttpResponse::Unauthorized().json(
                    serde_json::json!({
                        "error": "Unauthorized",
                        "message": "Invalid Authorization header format"
                    })
                );
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid Authorization header format"
                })
            );
        }

        token_str = Some(auth_str.trim_start_matches("Bearer ").trim().to_string());
    }
    
    // Unwrap the token (we know it's Some at this point)
    let token = token_str.unwrap();

    // Check if the token is blacklisted
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
            return match e.kind() {
                ErrorKind::ExpiredSignature => {
                    warn!("Token expired");
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    )
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    )
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid token"
                        })
                    )
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

    let role = claims.user_role.unwrap_or_else(|| "user".to_string());

    // Create AuthUser and insert into request extensions
    let auth_user = AuthUser {
        id: user_id,
        email: claims.email,
        role: role.clone(),
        name: claims.name,
        profile_image: claims.profile_image,
    };
    
    // Add the AuthUser to request extensions so the existing code can use it
    req.extensions_mut().insert(auth_user.clone());
    
    // Check if the user has the admin role
    if auth_user.role.to_lowercase() != "admin" {
        return HttpResponse::Forbidden().json(
            serde_json::json!({
                "error": "Forbidden",
                "message": "Admin role required for this resource"
            })
        );
    }
    
    debug!("Admin user {:?} is fetching all users", auth_user);
    
    // Fetch all users from the database
    match User::find().all(db.get_ref()).await {
        Ok(users) => {
            // Convert to DTOs to avoid exposing sensitive fields
            let user_dtos: Vec<UserResponseDto> = users.into_iter().map(|u| u.into()).collect();
            HttpResponse::Ok().json(user_dtos)
        }
        Err(e) => {
            error!("Database error when fetching users: {:?}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({
                    "error": "Database error",
                    "message": "Failed to retrieve users"
                })
            )
        }
    }
}

// Delete a user
#[delete("/api/admin/users/{user_id}")]
pub async fn delete_user(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for delete user endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for delete user endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for delete user endpoint");
                return HttpResponse::Unauthorized().json(
                    serde_json::json!({
                        "error": "Unauthorized",
                        "message": "Authentication required"
                    })
                );
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(e) => {
                warn!("Invalid Authorization header format: {:?}", e);
                return HttpResponse::Unauthorized().json(
                    serde_json::json!({
                        "error": "Unauthorized",
                        "message": "Invalid Authorization header format"
                    })
                );
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid Authorization header format"
                })
            );
        }

        token_str = Some(auth_str.trim_start_matches("Bearer ").trim().to_string());
    }
    
    // Unwrap the token (we know it's Some at this point)
    let token = token_str.unwrap();

    // Check if the token is blacklisted
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
            return match e.kind() {
                ErrorKind::ExpiredSignature => {
                    warn!("Token expired");
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    )
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    )
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid token"
                        })
                    )
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

    let role = claims.user_role.unwrap_or_else(|| "user".to_string());

    // Create AuthUser and insert into request extensions
    let auth_user = AuthUser {
        id: user_id,
        email: claims.email,
        role: role.clone(),
        name: claims.name,
        profile_image: claims.profile_image,
    };
    
    // Add the AuthUser to request extensions so the existing code can use it
    req.extensions_mut().insert(auth_user.clone());
    
    // Check if the user has the admin role
    if auth_user.role.to_lowercase() != "admin" {
        warn!("Non-admin user {} attempted to delete a user", auth_user.id);
        return HttpResponse::Forbidden().json(
            serde_json::json!({
                "error": "Forbidden",
                "message": "Admin role required for this resource"
            })
        );
    }
    
    let user_id_str = path.into_inner();
    
    // Parse the user ID
    let user_id = match Uuid::parse_str(&user_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({
                    "error": "Invalid user ID format"
                })
            );
        }
    };
    
    // Prevent admins from deleting themselves
    if auth_user.id == user_id {
        return HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Cannot delete yourself",
                "message": "Admins cannot delete their own account"
            })
        );
    }
    
    debug!("Admin is deleting user with ID: {}", user_id);
    
    // Delete the user
    match User::delete_by_id(user_id).exec(db.get_ref()).await {
        Ok(res) => {
            if res.rows_affected == 0 {
                return HttpResponse::NotFound().json(
                    serde_json::json!({
                        "error": "User not found"
                    })
                );
            }
            
            info!("User {} deleted successfully", user_id);
            HttpResponse::Ok().json(
                serde_json::json!({
                    "success": true,
                    "message": "User deleted successfully"
                })
            )
        }
        Err(e) => {
            error!("Database error when deleting user {}: {:?}", user_id, e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({
                    "error": "Database error",
                    "message": "Failed to delete user"
                })
            )
        }
    }
}

// Change a user's role
#[put("/api/admin/users/{user_id}/role")]
pub async fn change_user_role(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Json<ChangeRoleRequest>,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for change user role endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for change user role endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for change user role endpoint");
                return HttpResponse::Unauthorized().json(
                    serde_json::json!({
                        "error": "Unauthorized",
                        "message": "Authentication required"
                    })
                );
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(e) => {
                warn!("Invalid Authorization header format: {:?}", e);
                return HttpResponse::Unauthorized().json(
                    serde_json::json!({
                        "error": "Unauthorized",
                        "message": "Invalid Authorization header format"
                    })
                );
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid Authorization header format"
                })
            );
        }

        token_str = Some(auth_str.trim_start_matches("Bearer ").trim().to_string());
    }
    
    // Unwrap the token (we know it's Some at this point)
    let token = token_str.unwrap();

    // Check if the token is blacklisted
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
            return match e.kind() {
                ErrorKind::ExpiredSignature => {
                    warn!("Token expired");
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    )
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    )
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid token"
                        })
                    )
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

    let role = claims.user_role.unwrap_or_else(|| "user".to_string());

    // Create AuthUser and insert into request extensions
    let auth_user = AuthUser {
        id: user_id,
        email: claims.email,
        role: role.clone(),
        name: claims.name,
        profile_image: claims.profile_image,
    };
    
    // Add the AuthUser to request extensions so the existing code can use it
    req.extensions_mut().insert(auth_user.clone());
    
    // Check if the user has the admin role
    if auth_user.role.to_lowercase() != "admin" {
        warn!("Non-admin user {} attempted to change a user's role", auth_user.id);
        return HttpResponse::Forbidden().json(
            serde_json::json!({
                "error": "Forbidden",
                "message": "Admin role required for this resource"
            })
        );
    }
    
    let user_id_str = path.into_inner();
    
    // Parse the user ID
    let user_id = match Uuid::parse_str(&user_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({
                    "error": "Invalid user ID format"
                })
            );
        }
    };
    
    // Validate the role
    let role = body.role.to_lowercase();
    if role != "user" && role != "admin" {
        return HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Invalid role",
                "message": "Role must be either 'user' or 'admin'"
            })
        );
    }
    
    // Prevent admins from changing their own role
    if auth_user.id == user_id {
        return HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Cannot change your own role",
                "message": "Admins cannot change their own role"
            })
        );
    }
    
    debug!("Admin is changing role of user {} to {}", user_id, role);
    
    // Find the user
    match User::find_by_id(user_id).one(db.get_ref()).await {
        Ok(Some(user)) => {
            // Update the user's role
            let mut user_active: UserActiveModel = user.into();
            user_active.role = Set(role.clone());
            
            match user_active.update(db.get_ref()).await {
                Ok(updated_user) => {
                    info!("User {} role changed to {}", user_id, role);
                    let user_dto: UserResponseDto = updated_user.into();
                    HttpResponse::Ok().json(user_dto)
                }
                Err(e) => {
                    error!("Database error when updating user {}: {:?}", user_id, e);
                    HttpResponse::InternalServerError().json(
                        serde_json::json!({
                            "error": "Database error",
                            "message": "Failed to update user role"
                        })
                    )
                }
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().json(
                serde_json::json!({
                    "error": "User not found"
                })
            )
        }
        Err(e) => {
            error!("Database error when fetching user {}: {:?}", user_id, e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({
                    "error": "Database error",
                    "message": "Failed to retrieve user"
                })
            )
        }
    }
}

// Toggle a user's active status
#[put("/api/admin/users/{user_id}/status")]
pub async fn toggle_user_active(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Json<ToggleActiveRequest>,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for toggle user status endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for toggle user status endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for toggle user status endpoint");
                return HttpResponse::Unauthorized().json(
                    serde_json::json!({
                        "error": "Unauthorized",
                        "message": "Authentication required"
                    })
                );
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(e) => {
                warn!("Invalid Authorization header format: {:?}", e);
                return HttpResponse::Unauthorized().json(
                    serde_json::json!({
                        "error": "Unauthorized",
                        "message": "Invalid Authorization header format"
                    })
                );
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return HttpResponse::Unauthorized().json(
                serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid Authorization header format"
                })
            );
        }

        token_str = Some(auth_str.trim_start_matches("Bearer ").trim().to_string());
    }
    
    // Unwrap the token (we know it's Some at this point)
    let token = token_str.unwrap();

    // Check if the token is blacklisted
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
            return match e.kind() {
                ErrorKind::ExpiredSignature => {
                    warn!("Token expired");
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    )
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    )
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid token"
                        })
                    )
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

    let role = claims.user_role.unwrap_or_else(|| "user".to_string());

    // Create AuthUser and insert into request extensions
    let auth_user = AuthUser {
        id: user_id,
        email: claims.email,
        role: role.clone(),
        name: claims.name,
        profile_image: claims.profile_image,
    };
    
    // Add the AuthUser to request extensions so the existing code can use it
    req.extensions_mut().insert(auth_user.clone());
    
    // Check if the user has the admin role
    if auth_user.role.to_lowercase() != "admin" {
        warn!("Non-admin user {} attempted to change a user's active status", auth_user.id);
        return HttpResponse::Forbidden().json(
            serde_json::json!({
                "error": "Forbidden",
                "message": "Admin role required for this resource"
            })
        );
    }
    
    let user_id_str = path.into_inner();
    
    // Parse the user ID
    let user_id = match Uuid::parse_str(&user_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(
                serde_json::json!({
                    "error": "Invalid user ID format"
                })
            );
        }
    };
    
    // Prevent admins from deactivating themselves
    if auth_user.id == user_id && !body.is_active {
        return HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Cannot deactivate yourself",
                "message": "Admins cannot deactivate their own account"
            })
        );
    }
    
    debug!("Admin is changing active status of user {} to {}", user_id, body.is_active);
    
    // Find the user
    match User::find_by_id(user_id).one(db.get_ref()).await {
        Ok(Some(user)) => {
            // Update the user's active status
            let mut user_active: UserActiveModel = user.into();
            user_active.is_active = Set(body.is_active);
            
            match user_active.update(db.get_ref()).await {
                Ok(updated_user) => {
                    info!("User {} active status changed to {}", user_id, body.is_active);
                    let user_dto: UserResponseDto = updated_user.into();
                    HttpResponse::Ok().json(user_dto)
                }
                Err(e) => {
                    error!("Database error when updating user {}: {:?}", user_id, e);
                    HttpResponse::InternalServerError().json(
                        serde_json::json!({
                            "error": "Database error",
                            "message": "Failed to update user active status"
                        })
                    )
                }
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().json(
                serde_json::json!({
                    "error": "User not found"
                })
            )
        }
        Err(e) => {
            error!("Database error when fetching user {}: {:?}", user_id, e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({
                    "error": "Database error",
                    "message": "Failed to retrieve user"
                })
            )
        }
    }
}
