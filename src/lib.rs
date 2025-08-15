pub mod api;
pub mod database;
pub mod engine;
pub mod errors;
pub mod handlers;
pub mod types;

// Re-export commonly used types for external testing
pub use handlers::BattleHandler;
pub use types::*;
pub use errors::ApiError;

#[cfg(test)]
mod tests;