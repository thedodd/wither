use thiserror::Error;

/// A `Result` type alias using `WitherError` instances as the error variant.
pub type Result<T> = std::result::Result<T, WitherError>;

/// Wither error variants.
#[derive(Debug, Error)]
pub enum WitherError {
    /// An error from the underlying `mongodb` driver.
    #[error("{0}")]
    Mongo(#[from] mongodb::error::Error),
    /// An error related to BSON OID construction and generation.
    #[error("{0}")]
    BsonOid(#[from] mongodb::bson::oid::Error),
    /// A BSON deserialization error.
    #[error("{0}")]
    BsonDe(#[from] mongodb::bson::de::Error),
    /// A BSON serialization error.
    #[error("{0}")]
    BsonSer(#[from] mongodb::bson::ser::Error),
    /// An error indicating that an ObjectId is required for the requested operation.
    #[error("Model must have an ObjectId for this operation.")]
    ModelIdRequiredForOperation,
    /// An error indicating that a model was serialized to a BSON variant other than a document.
    #[error("Serializing model to BSON failed to produce a Bson::Document, got type {0:?}")]
    ModelSerToDocument(mongodb::bson::spec::ElementType),
    /// An error indicating that the server failed to return a document after an update.
    #[error("Server failed to return the updated document. Update may have failed.")]
    ServerFailedToReturnUpdatedDoc,
    /// An error indicating that the server failed to return an ObjectId.
    #[error("Server failed to return ObjectId of updated document.")]
    ServerFailedToReturnObjectId,
    /// An error indicating that one of `$set` or `$unset` must be specified for a migration.
    #[error("One of '$set' or '$unset' must be specified.")]
    MigrationSetOrUnsetRequired,
}
