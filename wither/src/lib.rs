#![cfg_attr(feature="docinclude", feature(external_doc))]
#![cfg_attr(feature="docinclude", doc(include="../README.md"))]

#[macro_use(doc, bson)]
pub extern crate bson;
extern crate chrono;
#[macro_use]
extern crate log;
pub extern crate mongodb;
extern crate serde;

pub mod migration;
pub mod model;

// Expose lower symbols in the top level module.
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
    pub use ::migration::{
        Migrating,
        Migration,
    };
    pub use ::model::Model;
}
