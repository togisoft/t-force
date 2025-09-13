// Migration to create a default admin user
// This migration creates a default admin user with the following credentials:
// Email: admin@example.com
// Password: Admin123!
// Role: admin
//
// IMPORTANT: For security reasons, you should change the default admin password
// after first login or set the ADMIN_PASSWORD environment variable.

use sea_orm_migration::prelude::*;
use uuid::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};
use std::env;
use dotenv::dotenv;

#[derive(DeriveMigrationName)]
pub struct Migration;

fn uuid_expr(id: Uuid) -> SimpleExpr {
    Expr::cust_with_values(
        "$1::uuid",
        vec![Value::String(Some(Box::new(id.to_string())))],
    )
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Load environment variables from a .env file at the project root
        dotenv().ok();

        // --- Read admin credentials from environment variables ---
        // If a variable is not found, use a default value.
        let admin_email = env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
        let admin_name = env::var("ADMIN_NAME").unwrap_or_else(|_| "System Administrator".to_string());

        // --- Secure Password Handling ---
        let admin_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
            // If ADMIN_PASSWORD is not set, use a strong default password
            let default_password = "Admin123!";
            println!("\n\n!!! IMPORTANT: ADMIN_PASSWORD environment variable not found.");
            println!("!!! Using default admin password: {}", default_password);
            println!("!!! Please change this password after first login for security reasons.\n\n");
            default_password.to_string()
        });

        // Hash the password using Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(admin_password.as_bytes(), &salt)
            .map_err(|e| DbErr::Custom(format!("Failed to hash password: {}", e)))?
            .to_string();

        // Generate the UUID (this will work after the Cargo.toml fix)
        let admin_id = Uuid::new_v4();

        // Insert the admin user into the database with all required fields
        let insert_admin = Query::insert()
            .into_table(Users::Table)
            .columns([
                Users::Id,
                Users::Email,
                Users::Name,
                Users::Provider,
                Users::Role,
                Users::PasswordHash,
                Users::ProfileImage,
                Users::PasswordResetToken,
                Users::PasswordResetExpires,
                Users::CreatedAt,
                Users::UpdatedAt,
            ])
            .values_panic([
                uuid_expr(admin_id),
                admin_email.clone().into(), // Clone email for use in .on_conflict
                admin_name.into(),
                "local".into(), // Provider for local email/password accounts
                "admin".into(), // Set the role to 'admin'
                password_hash.into(),
                SimpleExpr::from(Value::String(None)), // profile_image (NULL)
                SimpleExpr::from(Value::String(None)), // password_reset_token (NULL)
                SimpleExpr::from(Value::ChronoDateTimeUtc(None)), // password_reset_expires (NULL timestamp)
                Expr::current_timestamp().into(), // Set created_at to current timestamp
                Expr::current_timestamp().into(), // Set updated_at to current timestamp
            ])
            // Do not insert if a user with this email already exists
            .on_conflict(
                OnConflict::column(Users::Email)
                    .do_nothing()
                    .to_owned()
            )
            .to_owned();

        manager.exec_stmt(insert_admin).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Load .env to get the correct email for deletion
        dotenv().ok();
        let admin_email = env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());

        // This 'down' migration will remove the admin user
        let delete_admin = Query::delete()
            .from_table(Users::Table)
            .and_where(Expr::col(Users::Email).eq(admin_email))
            .to_owned();

        manager.exec_stmt(delete_admin).await?;

        Ok(())
    }
}

// The table is actually named "users" (plural) in the database schema,
// so we need to use the same name for the enum to match
#[derive(Iden)]
enum Users {
    Table,
    Id,
    Email,
    Name,
    ProfileImage,
    Provider,
    Role,
    PasswordHash,
    PasswordResetToken,
    PasswordResetExpires,
    CreatedAt,
    UpdatedAt,
}