// Authentication module
// Provides JWT-based authentication with user registration, login, and token refresh

pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod password;
pub mod repository;
pub mod service;
pub mod token;

// Re-export commonly used types
pub use error::AuthError;
pub use handlers::{login_handler, me_handler, refresh_handler, register_handler};
pub use middleware::{AuthenticatedUser, RequireRole};
pub use models::{AuthResponse, LoginRequest, RefreshRequest, RegisterRequest, Role, User, UserResponse};
pub use service::AuthService;
