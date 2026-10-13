//! Image storage adapters (Cloudinary and in-memory mock).

pub mod cloudinary;
pub mod mock;

pub use cloudinary::{CloudinaryConfig, CloudinaryStorage};
pub use mock::MockImageStorage;
