#![cfg_attr(feature = "docinclude", doc = include_str!("../README.md"))]

// Re-exports //
pub use async_trait::async_trait;
pub use mongodb;
pub use mongodb::bson;
pub use mongodb::IndexModel;

pub use wither_derive::Model;
#[cfg(any(feature = "sync"))]
pub use wither_derive::ModelSync;

// Common //
mod error;
pub use error::{Result, WitherError};

// Async //
mod cursor;
pub use cursor::ModelCursor;

mod migration;
pub use migration::{IntervalMigration, Migration};
mod model;
pub use model::Model;

/// All traits needed for basic usage of the wither system.
pub mod prelude {
    pub use crate::migration::{Migrating, Migration};
    pub use crate::model::Model;
    pub use wither_derive::Model;
}
