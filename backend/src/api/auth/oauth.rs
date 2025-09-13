use actix_web::{get, web, HttpResponse, Responder, HttpRequest};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenUrl, AuthorizationCode, TokenResponse,
};
use oauth2::reqwest::async_http_client;
use serde::{Deserialize, Serialize};
use std::env;
use log::{debug, error, info};
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait};
use uuid::Uuid;
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use std::collections::HashMap;

use crate::models::{User, UserActiveModel, entities::user::Column};
use crate::auth::Claims;

// OAuth provider enum
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum OAuthProvider {
    Google,
    GitHub,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::GitHub => "github",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "google" => Some(OAuthProvider::Google),
            "github" => Some(OAuthProvider::GitHub),
            _ => None,
        }
    }
}

// OAuth user info
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub profile_image: Option<String>,
    pub provider: String,
}

// OAuth callback query parameters
#[derive(Debug, Deserialize)]
pub struct OAuthCallback {
    code: String,
    state: String,
}

// In-memory state storage (in production, use Redis or database)
use std::sync::Mutex;
use std::sync::Arc;

lazy_static::lazy_static! {
    static ref OAUTH_STATES: Arc<Mutex<HashMap<String, OAuthProvider>>> = Arc::new(Mutex::new(HashMap::new()));
}

fn store_oauth_state(csrf_token: &str, provider: OAuthProvider) {
    let mut states = OAUTH_STATES.lock().unwrap();
    states.insert(csrf_token.to_string(), provider);
}

fn get_and_remove_oauth_state(csrf_token: &str) -> Option<OAuthProvider> {
    let mut states = OAUTH_STATES.lock().unwrap();
    states.remove(csrf_token)
}

// Create OAuth clients
fn create_google_oauth_client() -> Result<BasicClient, String> {
    let google_client_id = match env::var("GOOGLE_CLIENT_ID") {
        Ok(id) => {
            if id == "your-google-client-id" || id.is_empty() {
                return Err("GOOGLE_CLIENT_ID is set to a placeholder value. Please replace it with your actual Google Client ID in the .env file.".to_string());
            }
            id
        },
        Err(_) => {
            return Err("Missing GOOGLE_CLIENT_ID environment variable. Please set it in your .env file.".to_string());
        }
    };

    let google_client_secret = match env::var("GOOGLE_CLIENT_SECRET") {
        Ok(secret) => {
            if secret == "your-google-client-secret" || secret.is_empty() {
                return Err("GOOGLE_CLIENT_SECRET is set to a placeholder value. Please replace it with your actual Google Client Secret in the .env file.".to_string());
            }
            secret
        },
        Err(_) => {
            return Err("Missing GOOGLE_CLIENT_SECRET environment variable. Please set it in your .env file.".to_string());
        }
    };

    let redirect_url = env::var("OAUTH_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:8080/api/auth/oauth/callback".to_string());

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .map_err(|_| "Invalid Google authorization endpoint URL".to_string())?;

    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .map_err(|_| "Invalid Google token endpoint URL".to_string())?;

    let redirect_uri = RedirectUrl::new(redirect_url)
        .map_err(|_| "Invalid redirect URL".to_string())?;

    Ok(BasicClient::new(
        ClientId::new(google_client_id),
        Some(ClientSecret::new(google_client_secret)),
        auth_url,
        Some(token_url)
    )
        .set_redirect_uri(redirect_uri))
}

fn create_github_oauth_client() -> Result<BasicClient, String> {
    let github_client_id = match env::var("GITHUB_CLIENT_ID") {
        Ok(id) => {
            if id == "your-github-client-id" || id.is_empty() {
                return Err("GITHUB_CLIENT_ID is set to a placeholder value. Please replace it with your actual GitHub Client ID in the .env file.".to_string());
            }
            id
        },
        Err(_) => {
            return Err("Missing GITHUB_CLIENT_ID environment variable. Please set it in your .env file.".to_string());
        }
    };

    let github_client_secret = match env::var("GITHUB_CLIENT_SECRET") {
        Ok(secret) => {
            if secret == "your-github-client-secret" || secret.is_empty() {
                return Err("GITHUB_CLIENT_SECRET is set to a placeholder value. Please replace it with your actual GitHub Client Secret in the .env file.".to_string());
            }
            secret
        },
        Err(_) => {
            return Err("Missing GITHUB_CLIENT_SECRET environment variable. Please set it in your .env file.".to_string());
        }
    };

    let redirect_url = env::var("OAUTH_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:8080/api/auth/oauth/callback".to_string());

    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .map_err(|_| "Invalid GitHub authorization endpoint URL".to_string())?;

    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .map_err(|_| "Invalid GitHub token endpoint URL".to_string())?;

    let redirect_uri = RedirectUrl::new(redirect_url)
        .map_err(|_| "Invalid redirect URL".to_string())?;

    Ok(BasicClient::new(
        ClientId::new(github_client_id),
        Some(ClientSecret::new(github_client_secret)),
        auth_url,
        Some(token_url)
    )
        .set_redirect_uri(redirect_uri))
}

// OAuth login endpoint for Google
#[get("/api/auth/oauth/google")]
pub async fn oauth_google_login(_req: HttpRequest) -> impl Responder {
    debug!("Starting Google OAuth login flow");

    let client = match create_google_oauth_client() {
        Ok(client) => client,
        Err(error) => {
            error!("Failed to create Google OAuth client: {}", error);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({
                    "error": "OAuth Configuration Error",
                    "message": error,
                    "details": "Please check your .env file and ensure GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET are properly configured."
                })
            );
        }
    };

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    // Store the state in memory with provider information
    store_oauth_state(csrf_token.secret(), OAuthProvider::Google);

    debug!("Redirecting to Google OAuth authorization URL with state: {}", csrf_token.secret());
    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

// OAuth login endpoint for GitHub
#[get("/api/auth/oauth/github")]
pub async fn oauth_github_login(_req: HttpRequest) -> impl Responder {
    debug!("Starting GitHub OAuth login flow");

    let client = match create_github_oauth_client() {
        Ok(client) => client,
        Err(error) => {
            error!("Failed to create GitHub OAuth client: {}", error);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({
                    "error": "OAuth Configuration Error",
                    "message": error,
                    "details": "Please check your .env file and ensure GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET are properly configured."
                })
            );
        }
    };

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    // Store the state in memory with provider information
    store_oauth_state(csrf_token.secret(), OAuthProvider::GitHub);

    debug!("Redirecting to GitHub OAuth authorization URL with state: {}", csrf_token.secret());
    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

// Helper function to generate OAuth success response
fn generate_oauth_success_response(
    user: crate::models::entities::user::Model,
    jwt_secret: web::Data<String>,
) -> HttpResponse {
    let now = Utc::now();
    let exp = (now + Duration::hours(24)).timestamp() as usize;
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

    let is_production = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()) == "production";
    let max_age_seconds = Duration::hours(24).num_seconds();

    let cookie = actix_web::cookie::Cookie::build("auth_token", token)
        .path("/")
        .http_only(true)
        .same_site(actix_web::cookie::SameSite::Lax)
        .secure(is_production)
        .max_age(actix_web::cookie::time::Duration::seconds(max_age_seconds))
        .finish();

    let frontend_url = env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let redirect_url = format!("{}/dashboard", frontend_url);

    debug!("OAuth authentication successful, redirecting to frontend with secure cookie");
    HttpResponse::Found()
        .cookie(cookie)
        .append_header(("Location", redirect_url))
        .finish()
}

// OAuth callback endpoint
#[get("/api/auth/oauth/callback")]
pub async fn oauth_callback(
    query: web::Query<OAuthCallback>,
    db: web::Data<DatabaseConnection>,
    jwt_secret: web::Data<String>,
) -> impl Responder {
    debug!("Received OAuth callback with state: {}", query.state);

    // Retrieve and validate the state
    let provider = match get_and_remove_oauth_state(&query.state) {
        Some(provider) => provider,
        None => {
            error!("Invalid or expired OAuth state: {}", query.state);
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Invalid or expired authentication state. Please try again."})
            );
        }
    };

    debug!("OAuth callback for provider: {:?}", provider);

    let user_info = match provider {
        OAuthProvider::Google => {
            get_google_user_info(&query.code).await
        },
        OAuthProvider::GitHub => {
            get_github_user_info(&query.code).await
        }
    };

    let user_info = match user_info {
        Ok(info) => info,
        Err(e) => {
            error!("Failed to get user info: {:?}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to get user information from OAuth provider"})
            );
        }
    };

    let user = match find_or_create_user(db.get_ref(), user_info).await {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to find or create user: {:?}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to process user information"})
            );
        }
    };

    // Check if 2FA is enabled for the user
    use crate::models::entities::two_factor_auth::{Entity as TwoFactorAuth, Column as TwoFactorColumn};

    let two_factor = match TwoFactorAuth::find()
        .filter(TwoFactorColumn::UserId.eq(user.id))
        .one(db.get_ref())
        .await {
        Ok(Some(two_factor)) => two_factor,
        Ok(None) => {
            debug!("No 2FA record found for OAuth user: {}", user.id);
            return generate_oauth_success_response(user, jwt_secret);
        }
        Err(e) => {
            error!("Database error when finding 2FA record: {:?}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Database error when checking 2FA status"})
            );
        }
    };

    if two_factor.enabled {
        debug!("2FA is enabled for OAuth user: {}, requiring verification", user.id);

        let now = Utc::now();
        let exp = (now + Duration::minutes(5)).timestamp() as usize;
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

        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let redirect_url = format!("{}/oauth/callback?token={}&requires2fa=true", frontend_url, temp_token);

        debug!("OAuth user requires 2FA verification: {}, redirecting to 2FA page", user.id);
        HttpResponse::Found()
            .append_header(("Location", redirect_url))
            .finish()
    } else {
        debug!("2FA is not enabled for OAuth user: {}", user.id);
        generate_oauth_success_response(user, jwt_secret)
    }
}

async fn get_google_user_info(code: &str) -> Result<OAuthUserInfo, String> {
    debug!("Getting user info from Google");

    let client = create_google_oauth_client()
        .map_err(|error| format!("OAuth Configuration Error: {}", error))?;

    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(async_http_client)
        .await
        .map_err(|e| format!("Failed to exchange authorization code: {:?}", e))?;

    let access_token = token_result.access_token().secret();

    let user_info_url = "https://www.googleapis.com/oauth2/v2/userinfo";
    let user_info_response = reqwest::Client::new()
        .get(user_info_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Failed to get user info: {:?}", e))?;

    if !user_info_response.status().is_success() {
        return Err(format!("Failed to get user info: {}", user_info_response.status()));
    }

    #[derive(Deserialize)]
    struct GoogleUserInfo {
        id: String,
        email: String,
        name: String,
        picture: Option<String>,
    }

    let google_user: GoogleUserInfo = user_info_response.json()
        .await
        .map_err(|e| format!("Failed to parse user info: {:?}", e))?;

    Ok(OAuthUserInfo {
        id: google_user.id,
        email: google_user.email,
        name: google_user.name,
        profile_image: google_user.picture,
        provider: OAuthProvider::Google.as_str().to_string(),
    })
}

async fn get_github_user_info(code: &str) -> Result<OAuthUserInfo, String> {
    debug!("Getting user info from GitHub");

    let client = create_github_oauth_client()
        .map_err(|error| format!("OAuth Configuration Error: {}", error))?;

    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(async_http_client)
        .await
        .map_err(|e| format!("Failed to exchange authorization code: {:?}", e))?;

    let access_token = token_result.access_token().secret();

    let user_info_url = "https://api.github.com/user";
    let user_info_response = reqwest::Client::new()
        .get(user_info_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "T-Force")
        .send()
        .await
        .map_err(|e| format!("Failed to get user info: {:?}", e))?;

    if !user_info_response.status().is_success() {
        return Err(format!("Failed to get user info: {}", user_info_response.status()));
    }

    #[derive(Deserialize)]
    struct GitHubUserInfo {
        id: i64,
        login: String,
        name: Option<String>,
        avatar_url: Option<String>,
    }

    let github_user: GitHubUserInfo = user_info_response.json()
        .await
        .map_err(|e| format!("Failed to parse user info: {:?}", e))?;

    let email_url = "https://api.github.com/user/emails";
    let email_response = reqwest::Client::new()
        .get(email_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "T-Force")
        .send()
        .await
        .map_err(|e| format!("Failed to get user email: {:?}", e))?;

    if !email_response.status().is_success() {
        return Err(format!("Failed to get user email: {}", email_response.status()));
    }

    #[derive(Deserialize)]
    struct GitHubEmail {
        email: String,
        primary: bool,
        verified: bool,
    }

    let emails: Vec<GitHubEmail> = email_response.json()
        .await
        .map_err(|e| format!("Failed to parse email info: {:?}", e))?;

    let email = emails.iter()
        .find(|e| e.primary && e.verified)
        .or_else(|| emails.iter().find(|e| e.verified))
        .map(|e| e.email.clone())
        .ok_or_else(|| "No verified email found".to_string())?;

    Ok(OAuthUserInfo {
        id: github_user.id.to_string(),
        email,
        name: github_user.name.unwrap_or_else(|| github_user.login.clone()),
        profile_image: github_user.avatar_url,
        provider: OAuthProvider::GitHub.as_str().to_string(),
    })
}

async fn find_or_create_user(
    db: &DatabaseConnection,
    user_info: OAuthUserInfo,
) -> Result<crate::models::entities::user::Model, String> {
    debug!("Finding or creating user with email: {}", user_info.email);

    let user = User::find()
        .filter(Column::Email.eq(&user_info.email))
        .one(db)
        .await
        .map_err(|e| format!("Database error when finding user: {:?}", e))?;

    match user {
        Some(existing_user) => {
            debug!("User found with ID: {}", existing_user.id);
            let mut user_active: UserActiveModel = existing_user.clone().into();
            let mut has_changes = false;

            if existing_user.name != user_info.name {
                debug!("Updating user name from '{}' to '{}'", existing_user.name, user_info.name);
                user_active.name = Set(user_info.name);
                has_changes = true;
            }

            if existing_user.profile_image != user_info.profile_image {
                debug!("Updating user profile image");
                user_active.profile_image = Set(user_info.profile_image);
                has_changes = true;
            }

            if has_changes {
                debug!("User has changes, updating in database");
                let updated_user = user_active.update(db)
                    .await
                    .map_err(|e| format!("Failed to update user: {:?}", e))?;

                info!("User updated successfully: {}", updated_user.id);
                Ok(updated_user)
            } else {
                debug!("No changes needed for user: {}", existing_user.id);
                Ok(existing_user)
            }
        }
        None => {
            debug!("User not found, creating new user with email: {}", user_info.email);
            let user_id = Uuid::new_v4();
            let new_user = UserActiveModel {
                id: Set(user_id),
                email: Set(user_info.email.clone()),
                name: Set(user_info.name.clone()),
                profile_image: Set(user_info.profile_image.clone()),
                provider: Set(user_info.provider.clone()),
                role: Set("user".to_string()),
                password_hash: Set(None),
                password_reset_token: Set(None),
                password_reset_expires: Set(None),
                created_at: Set(Utc::now()),
                is_active: Set(true),
                updated_at: Set(Utc::now()),
            };

            let created_user = new_user.insert(db)
                .await
                .map_err(|e| format!("Failed to create user: {:?}", e))?;

            info!("New user created successfully: {}", created_user.id);
            Ok(created_user)
        }
    }
}