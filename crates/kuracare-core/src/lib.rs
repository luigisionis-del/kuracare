//! Kuracare core: encrypted on-device diary for self-care.
//!
//! No networking, no UI. Everything personal is sealed with AES-256-GCM
//! under a key wrapped by an Argon2id-derived key (see `crypto`).

pub mod crypto;
pub mod enrich;
pub mod error;
pub mod event;
pub mod export;
pub mod kinds;
pub mod lists;
pub mod migrations;
pub mod questionnaires;
pub mod recognition;
pub mod reminders;
pub mod repo;
pub mod settings;
pub mod store;
pub mod summary;
pub mod time;
pub mod vault;

pub use error::{Error, Result};
pub use event::{Event, Origin};
pub use kinds::Payload;
pub use store::{MemStore, SqliteStore, Store};
pub use vault::Vault;
