use crate::bson::Document;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct IndexModel {
    /// The fields to index, along with their sort order.
    pub keys: Document,
    /// Extra options to use when creating the index.
    pub options: Option<Document>,
}

impl IndexModel {
    /// Construct a new instance.
    pub fn new(keys: Document, options: Option<Document>) -> Self {
        Self{keys, options}
    }
}
