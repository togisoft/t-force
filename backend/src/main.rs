mod api;
mod auth;
mod models;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::Logger};

use dotenv::dotenv;
use std::env;

use migration::{Migrator, MigratorTrait, sea_orm::{Database, DatabaseConnection}};

// Import API handlers explicitly
use crate::api::auth::sync::sync_user;
use crate::api::auth::logout::logout_get;
use crate::api::auth::{
    login_handler, register_handler, logout_handler, validate_session,
    oauth_google_login, oauth_github_login, oauth_callback,
    two_factor_setup, two_factor_verify, two_factor_status, 
    two_factor_disable, two_factor_backup_codes, two_factor_regenerate_backup_codes,
    verify_two_factor_handler,
    get_sessions, terminate_session, terminate_all_sessions,
    forgot_password, reset_password
};
use crate::api::user::me::get_current_user;
use crate::api::user::{upload_profile_picture, get_profile_image, update_username, update_password};
use crate::api::basic::{root, health_check};
use crate::api::admin::users::{get_all_users, delete_user, change_user_role, toggle_user_active};
use crate::api::chat::{ws_index, create_room, get_rooms, get_room, delete_room, leave_room_membership, send_message, get_messages, verify_room_password, upload_chat_image, get_chat_image, join_room_by_code_handler, upload_voice_message, get_voice_message, upload_chat_video, get_chat_video};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Try to load environment variables from .env file
    if dotenv().is_err() {
        // If .env file is not found, try .env.docker
        if std::path::Path::new(".env.docker").exists() {
            dotenv::from_filename(".env.docker").ok();
            log::info!("Loaded environment variables from .env.docker");
        } else {
            log::warn!("No .env or .env.docker file found. Using environment variables from the system.");
        }
    } else {
        log::info!("Loaded environment variables from .env");
    }
    
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Get environment variables with better error messages
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            log::warn!("DATABASE_URL not set, using default value");
            "postgres://postgres:postgres@db:5432/tforce".to_string()
        });
    
    let jwt_secret = env::var("NEXTAUTH_SECRET")
        .unwrap_or_else(|_| {
            log::warn!("NEXTAUTH_SECRET not set, using a default value (not secure for production)");
            "insecure_default_secret_only_for_development".to_string()
        });
    
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server_url = format!("{}:{}", host, port);
    
    // Connect to the database
    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    // Run migrations
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    
    log::info!("Starting server at http://{}", server_url);
    
    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS based on environment
        let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string());
        
        let cors = if cors_origin == "*" {
            // Development mode - permissive CORS
            log::warn!("Using permissive CORS (allow_any_origin) - not recommended for production");
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .supports_credentials()
                .max_age(3600)
        } else {
            // Production mode - specific origin
            log::info!("Using production CORS with origin: {}", cors_origin);
            Cors::default()
                .allowed_origin(&cors_origin)
                .allow_any_method()
                .allow_any_header()
                .supports_credentials()
                .max_age(3600)
        };
        
        // JWT secret is passed to the app as app_data
        // Each route handler that needs authentication will use it
        
        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))
            // Register all routes directly
            .service(sync_user)
            .service(login_handler)
            .service(verify_two_factor_handler)
            .service(register_handler)
            .service(logout_handler)
            .service(logout_get)
            .service(validate_session)
            .service(get_current_user)
            // Sessions endpoints
            .service(get_sessions)
            .service(terminate_session)
            .service(terminate_all_sessions)
            // Profile endpoints
            .service(upload_profile_picture)
            .service(get_profile_image)
            .service(update_username)
            .service(update_password)
            // OAuth endpoints
            .service(oauth_google_login)
            .service(oauth_github_login)
            .service(oauth_callback)
            // 2FA endpoints
            .service(two_factor_setup)
            .service(two_factor_verify)
            .service(two_factor_status)
            .service(two_factor_disable)
            .service(two_factor_backup_codes)
            .service(two_factor_regenerate_backup_codes)
            // Basic endpoints
            .service(root)
            .service(health_check)
            // Admin endpoints
            .service(get_all_users)
            .service(delete_user)
            .service(change_user_role)
            .service(toggle_user_active)
            // Password reset endpoints
            .service(forgot_password)
            .service(reset_password)
            // Chat endpoints
            .service(ws_index)
            .service(create_room)
            .service(get_rooms)
            .service(get_room)
            .service(delete_room)
            .service(leave_room_membership)
            .service(send_message)
            .service(get_messages)
            .service(verify_room_password)
            .service(upload_chat_image)
            .service(get_chat_image)
            .service(join_room_by_code_handler)
            .service(upload_voice_message)
            .service(get_voice_message)
            .service(upload_chat_video)
            .service(get_chat_video)
    })
    .bind(server_url)?
    .run()
    .await
}