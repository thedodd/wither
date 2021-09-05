#![cfg_attr(feature = "docinclude", feature(external_doc))]
#![cfg_attr(feature = "docinclude", doc(include = "../README.md"))]

// Re-exports //
pub use async_trait::async_trait;

#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
pub use mongodb;
#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
pub use mongodb::bson;

#[cfg(any(feature = "tokio-beta-runtime", feature = "async-beta-std-runtime"))]
pub use mongodb_beta;
#[cfg(any(feature = "tokio-beta-runtime", feature = "async-beta-std-runtime"))]
pub use mongodb_beta::bson;


pub use wither_derive::Model;
#[cfg(any(feature = "sync"))]
pub use wither_derive::ModelSync;

// Common //
mod error;
pub use error::{Result, WitherError};
mod common;
pub use common::IndexModel;

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
