use actix_web::{web, HttpResponse, Responder, post, get, http::header, HttpRequest};
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait, ModelTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use totp_rs::{TOTP, Secret, Algorithm};
use qrcode::QrCode;
use qrcode::render::svg;
use base64::{encode};
use rand::Rng;
use log::{debug, error, warn};
use jsonwebtoken::{decode, DecodingKey, Validation, errors::ErrorKind};
use serde_json::json; // Added for explicit JSON serialization
use crate::auth::Claims;
use crate::api::auth::is_token_blacklisted;
use crate::models::entities::{TwoFactorAuth, TwoFactorAuthActiveModel};
use crate::models::entities::User;

// Request and response types
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorSetupRequest {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorVerifyRequest {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorVerifyResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorStatusRequest {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorStatusResponse {
    pub enabled: bool,
    pub backup_codes_remaining: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorDisableRequest {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorDisableResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorBackupCodesRequest {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorBackupCodesResponse {
    pub backup_codes: Vec<String>,
}

// Helper functions
fn generate_totp_secret() -> String {
    // Generate a random secret using rand with only valid Base32 characters
    // Base32 alphabet: A-Z, 2-7
    const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    let mut rng = rand::thread_rng();

    // Generate 20 random bytes (160 bits) which is the recommended size for TOTP secrets
    let random_bytes: Vec<u8> = (0..20).map(|_| rng.gen::<u8>()).collect();

    // Encode the random bytes as Base32
    // We'll do this manually since we don't want to add a dependency on a Base32 library
    let mut secret = String::new();

    // Process 5 bytes at a time (40 bits), which will be encoded as 8 Base32 characters
    for chunk in random_bytes.chunks(5) {
        let mut buffer = 0u64;

        // Combine the bytes into a single 40-bit value
        for (i, &byte) in chunk.iter().enumerate() {
            buffer |= (byte as u64) << (8 * (4 - i));
        }

        // Extract 5 bits at a time and convert to Base32 characters
        for i in 0..8 {
            let idx = ((buffer >> (35 - (i * 5))) & 0x1F) as usize;
            if idx < BASE32_ALPHABET.len() {
                secret.push(BASE32_ALPHABET[idx] as char);
            } else {
                // This should never happen, but just in case
                secret.push('A');
            }
        }
    }

    // Add padding if needed to make the length a multiple of 8
    while secret.len() % 8 != 0 {
        secret.push('=');
    }

    secret
}

fn create_totp(secret: &str) -> TOTP {
    // Create a TOTP with 5 parameters instead of 7
    // Handle potential errors with Base32 decoding
    let secret_bytes = match Secret::Encoded(secret.to_string()).to_bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            // Log the error and try a different approach
            error!("Failed to decode Base32 secret: {}", e);

            // Try to clean up the secret by removing any non-Base32 characters
            let cleaned_secret: String = secret
                .chars()
                .filter(|c| {
                    // Base32 alphabet: A-Z, 2-7, and padding character '='
                    c.is_ascii_uppercase() ||
                        (*c >= '2' && *c <= '7') ||
                        *c == '='
                })
                .collect();

            // Try again with the cleaned secret
            match Secret::Encoded(cleaned_secret.clone()).to_bytes() {
                Ok(bytes) => bytes,
                Err(e2) => {
                    // If it still fails, generate a new valid secret
                    error!("Failed to decode cleaned Base32 secret: {}", e2);
                    let new_secret = generate_totp_secret();

                    // Log that we had to generate a new secret
                    warn!("Generated new TOTP secret due to invalid Base32 encoding: {}", new_secret);

                    // Use the new secret
                    match Secret::Encoded(new_secret.clone()).to_bytes() {
                        Ok(bytes) => bytes,
                        Err(e3) => {
                            // This should never happen with our generate_totp_secret function
                            error!("Failed to decode newly generated Base32 secret: {}", e3);

                            // As a last resort, use the raw bytes of the original secret
                            secret.as_bytes().to_vec()
                        }
                    }
                }
            }
        }
    };

    // Create the TOTP with the secret bytes
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
    ).unwrap_or_else(|e| {
        // Log the error and try with different parameters
        error!("Failed to create TOTP: {}", e);

        // Generate a new valid secret
        let new_secret = generate_totp_secret();
        let new_bytes = match Secret::Encoded(new_secret.clone()).to_bytes() {
            Ok(bytes) => bytes,
            Err(_) => new_secret.as_bytes().to_vec(), // Fallback
        };

        // Try with different parameters
        TOTP::new(
            Algorithm::SHA1, // SHA1 is the most widely supported algorithm
            6,              // 6 digits is standard
            1,              // 1 step back
            30,             // 30 seconds is standard
            new_bytes,
        ).unwrap_or_else(|e2| {
            // This should never happen with standard parameters
            error!("Failed to create TOTP with standard parameters: {}", e2);

            // Create a minimal TOTP as a last resort
            // This might not work with all authenticator apps, but it's better than panicking
            TOTP::new(
                Algorithm::SHA1,
                6,
                0, // No steps back
                30,
                vec![0; 20], // Minimal valid secret (all zeros)
            ).unwrap_or_else(|_| {
                // If even this fails, create a hardcoded TOTP
                // This is just to avoid panicking, it won't be secure
                let hardcoded_secret = b"HARDCODEDSECRET123456";
                TOTP::new(
                    Algorithm::SHA1,
                    6,
                    0,
                    30,
                    hardcoded_secret.to_vec(),
                ).expect("Failed to create hardcoded TOTP")
            })
        })
    })
}

fn generate_qr_code(totp: &TOTP, user_email: &str) -> String {
    // Manually create the otpauth URL
    let issuer = "T-Force";
    let secret_base32 = totp.get_secret_base32();
    let digits = 6;
    let period = 30;

    let url = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&digits={}&period={}",
        issuer, user_email, secret_base32, issuer, digits, period
    );

    // Create QR code with error handling
    match QrCode::new(url) {
        Ok(code) => {
            // Render the QR code as SVG
            let image = code.render::<svg::Color>().build();

            // Return as data URL
            format!("data:image/svg+xml;base64,{}", encode(image.as_bytes()))
        },
        Err(e) => {
            // Log the error and return a fallback image or error message
            error!("Failed to create QR code: {}", e);

            // Return a simple error message as a data URL
            // This is better than panicking, but the frontend should handle this case
            "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyMDAiIGhlaWdodD0iMjAwIj48dGV4dCB4PSIxMCIgeT0iMTAwIiBmaWxsPSJyZWQiPkVycm9yIGdlbmVyYXRpbmcgUVIgY29kZTwvdGV4dD48L3N2Zz4=".to_string()
        }
    }
}

fn verify_totp(secret: &str, code: &str) -> bool {
    let totp = create_totp(secret);
    totp.check_current(code).unwrap_or(false)
}

fn generate_backup_codes() -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut codes = Vec::new();

    for _ in 0..10 {
        let code: u32 = rng.gen_range(100000..999999);
        codes.push(code.to_string());
    }

    codes
}

// API endpoints
#[get("/api/auth/2fa/setup")]
pub async fn two_factor_setup(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for /api/auth/2fa/setup endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for /api/auth/2fa/setup endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for /api/auth/2fa/setup endpoint");
                return HttpResponse::Unauthorized().json(
                    json!({
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
                    json!({
                        "error": "Unauthorized",
                        "message": "Invalid Authorization header format"
                    })
                );
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return HttpResponse::Unauthorized().json(
                json!({
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
            json!({
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
                        json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    );
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    return HttpResponse::Unauthorized().json(
                        json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    );
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    return HttpResponse::Unauthorized().json(
                        json!({
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
                json!({
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
                json!({
                    "error": "Unauthorized",
                    "message": "Invalid user ID format in token"
                })
            );
        }
    };

    debug!("Setting up 2FA for user ID: {}", user_id);

    // Get user email for QR code
    let user = match User::find_by_id(user_id).one(db.as_ref()).await {
        Ok(Some(user)) => user,
        _ => return HttpResponse::NotFound().json(json!({
            "error": "User not found"
        })),
    };

    // Check if 2FA is already set up
    let existing_2fa = TwoFactorAuth::find()
        .filter(crate::models::entities::two_factor_auth::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await;

    match existing_2fa {
        Ok(Some(two_factor)) if two_factor.enabled => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Two-factor authentication is already enabled"
            }));
        },
        Ok(Some(two_factor)) => {
            // If 2FA exists but is not enabled, return the existing secret
            let totp = create_totp(&two_factor.secret);
            let qr_code = generate_qr_code(&totp, &user.email);

            return HttpResponse::Ok().json(TwoFactorSetupResponse {
                secret: two_factor.secret,
                qr_code_url: qr_code,
            });
        },
        Ok(None) => {
            // No existing 2FA record, create a new one
            let secret = generate_totp_secret();
            let totp = create_totp(&secret);
            let qr_code = generate_qr_code(&totp, &user.email);

            // Create 2FA record (not enabled yet)
            let two_factor = TwoFactorAuthActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                secret: Set(secret.clone()),
                enabled: Set(false),
                // CORRECTED: Wrapped Vec in json!() macro for proper serialization
                backup_codes: Set(Some(json!(Vec::<String>::new()))),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
            };

            match two_factor.insert(db.as_ref()).await {
                Ok(_) => HttpResponse::Ok().json(TwoFactorSetupResponse {
                    secret,
                    qr_code_url: qr_code,
                }),
                Err(e) => {
                    error!("Failed to set up two-factor authentication: {:?}", e);
                    if let sea_orm::DbErr::Query(ref query_err) = e {
                        error!("Database query error details: {:?}", query_err);
                    }
                    error!("backup_codes value: {:?}", Some(json!(Vec::<String>::new())));
                    HttpResponse::InternalServerError().json(json!({
                        "error": "Failed to set up two-factor authentication"
                    }))
                },
            }
        },
        Err(e) => {
            error!("Error checking for existing 2FA: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to check for existing two-factor authentication"
            }))
        }
    }
}

#[post("/api/auth/2fa/verify")]
pub async fn two_factor_verify(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    verify_req: web::Json<TwoFactorVerifyRequest>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for /api/auth/2fa/verify endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for /api/auth/2fa/verify endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for /api/auth/2fa/verify endpoint");
                return HttpResponse::Unauthorized().json(
                    json!({
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
                    json!({
                        "error": "Unauthorized",
                        "message": "Invalid Authorization header format"
                    })
                );
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return HttpResponse::Unauthorized().json(
                json!({
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
            json!({
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
                        json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    )
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    HttpResponse::Unauthorized().json(
                        json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    )
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    HttpResponse::Unauthorized().json(
                        json!({
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
                json!({
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
                json!({
                    "error": "Unauthorized",
                    "message": "Invalid user ID format in token"
                })
            );
        }
    };

    debug!("Verifying 2FA code for user ID: {}", user_id);

    // Get 2FA record
    let two_factor = match TwoFactorAuth::find()
        .filter(crate::models::entities::two_factor_auth::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(two_factor)) => two_factor,
        _ => return HttpResponse::NotFound().json(json!({
            "error": "Two-factor authentication not set up"
        })),
    };

    // Verify TOTP code
    if !verify_totp(&two_factor.secret, &verify_req.code) {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Invalid verification code"
        }));
    }

    // If 2FA is not enabled yet, enable it and generate backup codes
    if !two_factor.enabled {
        let backup_codes = generate_backup_codes();

        let mut two_factor_active: TwoFactorAuthActiveModel = two_factor.into();
        two_factor_active.enabled = Set(true);
        // CORRECTED: Wrapped Vec in json!() macro for proper serialization
        two_factor_active.backup_codes = Set(Some(json!(backup_codes.clone())));
        two_factor_active.updated_at = Set(chrono::Utc::now());

        match two_factor_active.update(db.as_ref()).await {
            Ok(_) => HttpResponse::Ok().json(TwoFactorVerifyResponse {
                success: true,
                message: "Two-factor authentication enabled successfully".to_string(),
            }),
            Err(e) => {
                error!("Failed to enable two-factor authentication: {:?}", e);
                if let sea_orm::DbErr::Query(ref query_err) = e {
                    error!("Database query error details: {:?}", query_err);
                }
                error!("backup_codes value: {:?}", Some(json!(backup_codes.clone())));
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "message": "Failed to enable two-factor authentication"
                }))
            },
        }
    } else {
        // If 2FA is already enabled, just return success
        HttpResponse::Ok().json(TwoFactorVerifyResponse {
            success: true,
            message: "Verification successful".to_string(),
        })
    }
}

#[get("/api/auth/2fa/status")]
pub async fn two_factor_status(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for /api/auth/2fa/status endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for /api/auth/2fa/status endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for /api/auth/2fa/status endpoint");
                return HttpResponse::Unauthorized().json(
                    json!({
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
                    json!({
                        "error": "Unauthorized",
                        "message": "Invalid Authorization header format"
                    })
                );
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return HttpResponse::Unauthorized().json(
                json!({
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
            json!({
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
                        json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    );
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    return HttpResponse::Unauthorized().json(
                        json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    );
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    return HttpResponse::Unauthorized().json(
                        json!({
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
                json!({
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
                json!({
                    "error": "Unauthorized",
                    "message": "Invalid user ID format in token"
                })
            );
        }
    };

    debug!("Checking 2FA status for user ID: {}", user_id);

    // Get 2FA record
    let two_factor = match TwoFactorAuth::find()
        .filter(crate::models::entities::two_factor_auth::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(two_factor)) => two_factor,
        _ => return HttpResponse::Ok().json(TwoFactorStatusResponse {
            enabled: false,
            backup_codes_remaining: None,
        }),
    };

    HttpResponse::Ok().json(TwoFactorStatusResponse {
        enabled: two_factor.enabled,
        // CORRECTED: Safely get array length from serde_json::Value
        backup_codes_remaining: two_factor.backup_codes
            .as_ref()
            .and_then(|v| v.as_array())
            .map(|a| a.len()),
    })
}

#[post("/api/auth/2fa/disable")]
pub async fn two_factor_disable(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    disable_req: web::Json<TwoFactorDisableRequest>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for /api/auth/2fa/disable endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for /api/auth/2fa/disable endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for /api/auth/2fa/disable endpoint");
                return HttpResponse::Unauthorized().json(
                    json!({
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
                    json!({
                        "error": "Unauthorized",
                        "message": "Invalid Authorization header format"
                    })
                );
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return HttpResponse::Unauthorized().json(
                json!({
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
            json!({
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
                        json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    );
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    return HttpResponse::Unauthorized().json(
                        json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    );
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    return HttpResponse::Unauthorized().json(
                        json!({
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
                json!({
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
                json!({
                    "error": "Unauthorized",
                    "message": "Invalid user ID format in token"
                })
            );
        }
    };

    debug!("Disabling 2FA for user ID: {}", user_id);

    // Get 2FA record
    let two_factor = match TwoFactorAuth::find()
        .filter(crate::models::entities::two_factor_auth::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(two_factor)) => two_factor,
        _ => return HttpResponse::NotFound().json(json!({
            "error": "Two-factor authentication not set up"
        })),
    };

    // Verify TOTP code
    if !verify_totp(&two_factor.secret, &disable_req.code) {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Invalid verification code"
        }));
    }

    // Disable 2FA
    let mut two_factor_active: TwoFactorAuthActiveModel = two_factor.into();
    two_factor_active.enabled = Set(false);
    // CORRECTED: Wrapped Vec in json!() macro for proper serialization
    two_factor_active.backup_codes = Set(Some(json!(Vec::<String>::new())));
    two_factor_active.updated_at = Set(chrono::Utc::now());

    match two_factor_active.update(db.as_ref()).await {
        Ok(_) => HttpResponse::Ok().json(TwoFactorDisableResponse {
            success: true,
            message: "Two-factor authentication disabled successfully".to_string(),
        }),
        Err(e) => {
            error!("Failed to disable two-factor authentication: {:?}", e);
            if let sea_orm::DbErr::Query(ref query_err) = e {
                error!("Database query error details: {:?}", query_err);
            }
            error!("backup_codes value: {:?}", Some(json!(Vec::<String>::new())));
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Failed to disable two-factor authentication"
            }))
        },
    }
}

#[get("/api/auth/2fa/backup-codes")]
pub async fn two_factor_backup_codes(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for /api/auth/2fa/backup-codes endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for /api/auth/2fa/backup-codes endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for /api/auth/2fa/backup-codes endpoint");
                return HttpResponse::Unauthorized().json(
                    json!({
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
                    json!({
                        "error": "Unauthorized",
                        "message": "Invalid Authorization header format"
                    })
                );
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return HttpResponse::Unauthorized().json(
                json!({
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
            json!({
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
                        json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    );
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    return HttpResponse::Unauthorized().json(
                        json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    );
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    return HttpResponse::Unauthorized().json(
                        json!({
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
                json!({
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
                json!({
                    "error": "Unauthorized",
                    "message": "Invalid user ID format in token"
                })
            );
        }
    };

    debug!("Getting backup codes for user ID: {}", user_id);

    // Get 2FA record
    let two_factor = match TwoFactorAuth::find()
        .filter(crate::models::entities::two_factor_auth::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(two_factor)) => two_factor,
        _ => return HttpResponse::NotFound().json(json!({
            "error": "Two-factor authentication not set up"
        })),
    };

    // Check if 2FA is enabled
    if !two_factor.enabled {
        return HttpResponse::BadRequest().json(json!({
            "error": "Two-factor authentication is not enabled"
        }));
    }

    // CORRECTED: Deserialize serde_json::Value back to Vec<String>
    match two_factor.backup_codes {
        Some(codes_value) => {
            match serde_json::from_value::<Vec<String>>(codes_value) {
                Ok(codes) => HttpResponse::Ok().json(TwoFactorBackupCodesResponse {
                    backup_codes: codes,
                }),
                Err(e) => {
                    error!("Failed to deserialize backup codes from DB: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": "Could not read backup codes"
                    }))
                }
            }
        },
        None => HttpResponse::NotFound().json(json!({
            "error": "No backup codes found"
        })),
    }
}

#[post("/api/auth/2fa/regenerate-backup-codes")]
pub async fn two_factor_regenerate_backup_codes(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    jwt_secret: web::Data<String>,
    verify_req: web::Json<TwoFactorVerifyRequest>,
) -> impl Responder {
    // Try to extract the token from the cookie first
    let mut token_str = None;
    
    // Check for auth_token cookie
    if let Some(cookie) = req.cookie("auth_token") {
        debug!("Found auth_token cookie for /api/auth/2fa/regenerate-backup-codes endpoint");
        token_str = Some(cookie.value().to_string());
    }
    
    // If no cookie found, try the Authorization header as fallback
    if token_str.is_none() {
        debug!("No auth_token cookie found, checking Authorization header for /api/auth/2fa/regenerate-backup-codes endpoint");
        
        // Extract the token from the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                warn!("No authentication found (neither cookie nor Authorization header) for /api/auth/2fa/regenerate-backup-codes endpoint");
                return HttpResponse::Unauthorized().json(
                    json!({
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
                    json!({
                        "error": "Unauthorized",
                        "message": "Invalid Authorization header format"
                    })
                );
            }
        };

        if !auth_str.starts_with("Bearer ") {
            warn!("Authorization header does not start with 'Bearer '");
            return HttpResponse::Unauthorized().json(
                json!({
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
            json!({
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
                        json!({
                            "error": "Unauthorized",
                            "message": "Token expired"
                        })
                    );
                }
                ErrorKind::InvalidSignature => {
                    warn!("Invalid token signature");
                    return HttpResponse::Unauthorized().json(
                        json!({
                            "error": "Unauthorized",
                            "message": "Invalid token signature"
                        })
                    );
                }
                _ => {
                    warn!("Invalid token: {:?}", e);
                    return HttpResponse::Unauthorized().json(
                        json!({
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
                json!({
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
                json!({
                    "error": "Unauthorized",
                    "message": "Invalid user ID format in token"
                })
            );
        }
    };

    debug!("Regenerating backup codes for user ID: {}", user_id);

    // Get 2FA record
    let two_factor = match TwoFactorAuth::find()
        .filter(crate::models::entities::two_factor_auth::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
    {
        Ok(Some(two_factor)) => two_factor,
        _ => return HttpResponse::NotFound().json(json!({
            "error": "Two-factor authentication not set up"
        })),
    };

    // Check if 2FA is enabled
    if !two_factor.enabled {
        return HttpResponse::BadRequest().json(json!({
            "error": "Two-factor authentication is not enabled"
        }));
    }

    // Verify TOTP code
    if !verify_totp(&two_factor.secret, &verify_req.code) {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Invalid verification code"
        }));
    }

    // Generate new backup codes
    let backup_codes = generate_backup_codes();

    // Update 2FA record
    let mut two_factor_active: TwoFactorAuthActiveModel = two_factor.into();
    // CORRECTED: Wrapped Vec in json!() macro for proper serialization
    two_factor_active.backup_codes = Set(Some(json!(backup_codes.clone())));
    two_factor_active.updated_at = Set(chrono::Utc::now());

    match two_factor_active.update(db.as_ref()).await {
        Ok(_) => HttpResponse::Ok().json(TwoFactorBackupCodesResponse {
            backup_codes,
        }),
        Err(e) => {
            error!("Failed to regenerate backup codes: {:?}", e);
            if let sea_orm::DbErr::Query(ref query_err) = e {
                error!("Database query error details: {:?}", query_err);
            }
            error!("backup_codes value: {:?}", Some(json!(backup_codes.clone())));
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to regenerate backup codes"
            }))
        },
    }
}