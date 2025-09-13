pub mod sync;
pub mod login;
pub mod register;
pub mod logout;
pub mod validate;
pub mod oauth;
pub mod two_factor;
pub mod sessions;
pub mod password_reset;

pub use login::login as login_handler;
pub use login::verify_two_factor as verify_two_factor_handler;
pub use register::register as register_handler;
pub use logout::{logout as logout_handler, is_token_blacklisted};
pub use validate::validate_session;
pub use oauth::{oauth_google_login, oauth_github_login, oauth_callback};
pub use two_factor::{
    two_factor_setup, two_factor_verify, two_factor_status, 
    two_factor_disable, two_factor_backup_codes, two_factor_regenerate_backup_codes
};
pub use sessions::{
    get_sessions, terminate_session, terminate_all_sessions, create_session
};
pub use password_reset::{
    forgot_password, reset_password
};