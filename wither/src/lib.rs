#![cfg_attr(feature="docinclude", feature(external_doc))]
#![cfg_attr(feature="docinclude", doc(include="../README.md"))]

// Re-exports //
pub use mongodb;
pub use mongodb::bson;

pub use wither_derive::Model;
#[cfg(any(feature="sync"))]
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
pub use migration::{
    IntervalMigration,
    Migration,
};
mod model;
pub use model::Model;

// Sync //
#[cfg(any(feature="sync"))]
mod sync;
#[cfg(any(feature="sync"))]
pub use sync::ModelCursorSync;

#[cfg(any(feature="sync"))]
pub use sync::{
    IntervalMigrationSync,
    MigrationSync,
};
#[cfg(any(feature="sync"))]
pub use sync::ModelSync;

/// All traits needed for basic usage of the wither system.
pub mod prelude {
    pub use crate::migration::{
        Migrating,
        Migration,
    };
    pub use crate::model::Model;
    pub use wither_derive::Model;

    #[cfg(any(feature="sync"))]
    pub use crate::sync::{
        MigratingSync,
        MigrationSync,
    };
    #[cfg(any(feature="sync"))]
    pub use crate::sync::ModelSync;
    #[cfg(any(feature="sync"))]
    pub use wither_derive::ModelSync;
}
