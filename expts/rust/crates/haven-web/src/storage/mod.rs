//! Image storage adapters (Cloudinary and in-memory mock).

pub mod cloudinary;
pub mod mock;
pub mod outbox_worker;

pub use cloudinary::{CloudinaryConfig, CloudinaryStorage};
pub use mock::MockImageStorage;
pub use outbox_worker::{OutboxWorkerConfig, process_pending_purges, spawn_outbox_worker};
