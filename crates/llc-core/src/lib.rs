//! Platform-neutral configuration, validation, scheduling and persistence.
pub mod config;
pub mod engine;
pub mod persistence;
pub use config::*;

pub const MAX_CONFIG_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_PROFILES: usize = 256;
pub const MAX_MACROS: usize = 1024;
pub const MAX_STORED_OPS: usize = 4096;
pub const MAX_EXPANDED_OPS: usize = 100_000;
pub const MAX_DURATION_MS: u64 = 3_600_000;
pub const MAX_DELAY_MS: u64 = 600_000;
pub const MAX_REPEAT: u32 = 10_000;
pub const MAX_DEPTH: usize = 4;
pub const MAX_INSTANCES: usize = 32;
pub const MAX_EVENTS_PER_SECOND: usize = 2000;
pub const MIN_INTERVAL_MS: u64 = 10;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid configuration: {0}")]
    Invalid(String),
    #[error("revision conflict: current revision is {0}")]
    Revision(u64),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
pub(crate) fn invalid<T>(message: impl Into<String>) -> Result<T> {
    Err(Error::Invalid(message.into()))
}
