pub mod ws;
pub mod room;
pub mod message;
pub mod upload;
pub mod join_room_by_code;
pub mod voice;

// Re-export handlers for easier access
pub use ws::ws_index;
pub use room::{create_room, get_rooms, get_room, verify_room_password, delete_room, leave_room_membership};
pub use message::{send_message, get_messages};
pub use upload::{upload_chat_image, get_chat_image, upload_chat_video, get_chat_video};
pub use join_room_by_code::join_room_by_code as join_room_by_code_handler;
pub use voice::{upload_voice_message, get_voice_message};