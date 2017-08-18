#[macro_use(doc, bson)]
extern crate bson;
extern crate mongodb;
extern crate serde;

pub use bson::Document;
pub use mongodb::coll::options::FindOptions;

pub mod model;

// Expose lower symbols in the top level module.
pub use model::Model;
pub use model::basic_index_options;
