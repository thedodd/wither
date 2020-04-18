#![cfg_attr(feature="docinclude", feature(external_doc))]
#![cfg_attr(feature="docinclude", doc(include="../README.md"))]

////////////////
// Re-exports //
pub use bson;
pub use mongodb;

mod cursor;
pub mod migration;
pub mod model;

// Expose lower symbols in the top level module.
pub use cursor::ModelCursor;
pub use migration::{
    IntervalMigration,
    Migration,
};
pub use model::{
    basic_index_options,
    Model,
};

pub mod prelude {
    //! All traits needed for basic usage of the wither system.
    pub use crate::migration::{
        Migrating,
        Migration,
    };
    pub use crate::model::Model;
}
