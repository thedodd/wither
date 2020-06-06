#![cfg_attr(feature="docinclude", feature(external_doc))]
#![cfg_attr(feature="docinclude", doc(include="../README.md"))]

// Re-exports //
pub use mongodb;
pub use mongodb::bson;

#[cfg(not(feature="sync"))]
pub use wither_derive::Model;
#[cfg(any(feature="sync", feature="docinclude"))]
pub use wither_derive::ModelSync;

// Common //
mod error;
pub use error::{Result, WitherError};

// Async //
#[cfg(not(feature="sync"))]
mod cursor;
#[cfg(not(feature="sync"))]
pub use cursor::ModelCursor;

#[cfg(not(feature="sync"))]
mod migration;
#[cfg(not(feature="sync"))]
pub use migration::{
    IntervalMigration,
    Migration,
};
#[cfg(not(feature="sync"))]
mod model;
#[cfg(not(feature="sync"))]
pub use model::Model;

// Sync //
#[cfg(any(feature="sync", feature="docinclude"))]
mod sync;
#[cfg(any(feature="sync", feature="docinclude"))]
pub use sync::ModelCursorSync;

#[cfg(any(feature="sync", feature="docinclude"))]
pub use sync::{
    IntervalMigrationSync,
    MigrationSync,
};
#[cfg(any(feature="sync", feature="docinclude"))]
pub use sync::ModelSync;

/// All traits needed for basic usage of the wither system.
pub mod prelude {
    #[cfg(not(feature="sync"))]
    pub use crate::migration::{
        Migrating,
        Migration,
    };
    #[cfg(not(feature="sync"))]
    pub use crate::model::Model;
    #[cfg(not(feature="sync"))]
    pub use wither_derive::Model;

    #[cfg(any(feature="sync", feature="docinclude"))]
    pub use crate::sync::{
        MigratingSync,
        MigrationSync,
    };
    #[cfg(any(feature="sync", feature="docinclude"))]
    pub use crate::sync::ModelSync;
    #[cfg(any(feature="sync", feature="docinclude"))]
    pub use wither_derive::ModelSync;
}
