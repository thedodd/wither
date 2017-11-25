//! Interface for defining & using data models.
//!
//! `Model` is the central type in this crate. The entire purpose of this create is to simplify
//! the process of interfacing with MongoDB in Rust for patterns which should be simple. This
//! system allows you to define a data model using a normal struct, and then interact with your
//! MongoDB database collections using that struct.
//!
//! Implementing `Model` for your custom structs is quite simple.
//!
//! - define an associated constant `COLLECTION_NAME` in your impl which will be the name of the
//!   collection where the corresponding model's data will be read from & written to.
//! - provide an implementation for the `id`, `set_id` & `indexes` methods.
//!
//! That's it! Now you can easliy perform standard CRUD operations on MongoDB
//! using your models.

use std::collections::HashMap;
use std::error::Error;

use bson;
use bson::Document;
use bson::oid::ObjectId;
use mongodb::error::Error::{
    ArgumentError,
    DecoderError,
    DefaultError,
    OIDError,
    ResponseError,
};
use mongodb::error::Result;
use mongodb::coll::Collection;
use mongodb::coll::options::{
    CountOptions,
    FindOneAndUpdateOptions,
    FindOptions,
    IndexModel,
    IndexOptions,
    ReturnDocument,
};
use mongodb::common::WriteConcern;
use mongodb::db::{
    Database,
    ThreadedDatabase,
};
use serde::{
    Serialize,
    Deserialize,
};

use migration::Migration;

/// The name of the default index created by MongoDB.
pub const DEFAULT_INDEX: &str = "_id";

/// A convenience function for basic index options. Everything else will default to `None`.
pub fn basic_index_options(name: &str, background: bool, unique: Option<bool>, expire_after_seconds: Option<i32>, sparse: Option<bool>) -> IndexOptions {
    return IndexOptions{
        name: Some(name.to_owned()),
        background: Some(background),
        unique,
        expire_after_seconds,
        sparse,
        storage_engine: None,
        version: None,
        default_language: None,
        language_override: None,
        text_version: None,
        weights: None,
        sphere_version: None,
        bits: None,
        max: None,
        min: None,
        bucket_size: None,
    };
}

/// Model provides data modeling behaviors for interacting with MongoDB database collections.
pub trait Model<'a> where Self: Serialize + Deserialize<'a> {

    /// The name of the collection where this model's data is stored.
    const COLLECTION_NAME: &'static str;

    /// Get the ID for this model instance.
    fn id(&self) -> Option<ObjectId>;

    /// Set the ID for this model.
    fn set_id(&mut self, ObjectId);

    ///////////////////////////////
    // Write Concern Abstraction //

    /// The model's write concern for database writes.
    ///
    /// The default implementation ensures that all writes block until they are journaled, which
    /// ensures that an ObjectId will be returned for the inserted document. For most cases,
    /// overriding this implementation should be unnecessary.
    fn model_write_concern() -> WriteConcern {
        return WriteConcern{
            w: Self::write_concern_w(),
            w_timeout: Self::write_concern_w_timeout(),
            j: Self::write_concern_j(),
            fsync: Self::write_concern_fsync(),
        };
    }

    /// The write replication settings for this model. Defaults to `1`.
    fn write_concern_w() -> i32 {
        return 1;
    }

    /// The write concern timeout settings for this model. Defaults to `0`.
    fn write_concern_w_timeout() -> i32 {
        return 0;
    }

    /// The write concern journal settings for this model. Defaults to `true`.
    fn write_concern_j() -> bool {
        return true;
    }

    /// The write concern fsync settings for this model. Defaults to `false`.
    fn write_concern_fsync() -> bool {
        return false;
    }

    //////////////////
    // Static Layer //

    /// Count the number of documents in this model's collection matching the given criteria.
    fn count(db: Database, filter: Option<Document>, options: Option<CountOptions>) -> Result<i64> {
        let coll = db.collection(Self::COLLECTION_NAME);
        coll.count(filter, options)
    }

    /// Find all instances of this model matching the given query.
    fn find(db: Database, filter: Option<Document>, options: Option<FindOptions>) -> Result<Vec<Self>> {
        let coll = db.collection(Self::COLLECTION_NAME);

        // Unwrap cursor.
        let mut cursor = match coll.find(filter, options) {
            Ok(cursor) => cursor,
            Err(err) => return Err(err),
        };

        // Collect all items in the cursor.
        let bson_docs = match cursor.drain_current_batch() {
            Ok(docs) => docs,
            Err(err) => return Err(err),
        };

        // Deserialize bson docs onto struct models.
        let mut instances: Vec<Self> = vec![];
        for doc in bson_docs {
            let inst = Self::instance_from_document(doc)?;
            instances.push(inst);
        }
        Ok(instances)
    }

    /// Delete any model instances matching the given query.
    fn delete_many(db: Database, filter: Document) -> Result<()> {
        let coll = db.collection(Self::COLLECTION_NAME);
        coll.delete_many(filter, Some(Self::model_write_concern()))?;
        Ok(())
    }

    /// Find the one model record matching your query, returning a model instance.
    fn find_one(db: Database, filter: Option<Document>, options: Option<FindOptions>) -> Result<Option<Self>> {
        let coll = db.collection(Self::COLLECTION_NAME);

        // Unwrap result.
        let doc_option = match coll.find_one(filter, options) {
            Ok(doc_option) => doc_option,
            Err(err) => return Err(err),
        };

        // Unwrap option.
        let doc = match doc_option {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Deserialize bson onto struct model.
        let instance = Self::instance_from_document(doc)?;
        Ok(Some(instance))
    }

    ////////////////////
    // Instance Layer //

    /// Delete this model instance by ID.
    fn delete(&self, db: Database) -> Result<()> {
        // Return an error if the instance was never saved.
        let id = self.id().ok_or(DefaultError("This instance has no ID. Can not be deleted.".to_string()))?;

        let coll = db.collection(Self::COLLECTION_NAME);
        coll.delete_one(doc!{"_id": id}, Some(Self::model_write_concern()))?;
        Ok(())
    }

    /// Save the current model instance.
    ///
    /// In order to make this method as flexible as possible, its behavior varies a little based
    /// on the input and the state of the instance.
    ///
    /// When the instance already has an ID, this method will operate purely based on the instance
    /// ID. If no ID is present, and no `filter` has been specified, then an ID will be generated.
    ///
    /// If a `filter` is specified, and no ID exists for the instance, then the filter will be used
    /// and the first document matching the filter will be replaced by this instance. This is
    /// useful when the model has unique indexes on fields which need to be the target of the save
    /// operation.
    fn save(&mut self, db: Database, filter: Option<Document>) -> Result<()> {
        let coll = db.collection(Self::COLLECTION_NAME);
        let instance_doc = match bson::to_bson(&self)? {
            bson::Bson::Document(doc) => doc,
            _ => return Err(DefaultError("Failed to convert struct to a bson document.".to_string())),
        };

        // Ensure that journaling is set to true for this call, as we need to be able to get an ID back.
        let mut write_concern = Self::model_write_concern();
        write_concern.j = true;

        // Handle case where instance already has an ID.
        let mut id_needs_update = false;
        let _filter = if let Some(id) = self.id() {
            doc!{"_id" => id}

        // Handle case where no filter and no ID exist.
        } else if filter == None {
            let new_id = match ObjectId::new() {
                Ok(new) => new,
                Err(err) => return Err(OIDError(err)),
            };
            self.set_id(new_id.clone());
            doc!{"_id" => new_id}

        // Handle case where no ID exists, and a filter has been provided.
        } else {
            id_needs_update = true;
            filter.unwrap()
        };

        // Save the record by replacing it entirely, or upserting if it doesn't already exist.
        let opts = FindOneAndUpdateOptions{upsert: Some(true), write_concern: Some(write_concern), return_document: Some(ReturnDocument::After), sort: None, projection: None, max_time_ms: None};
        let updated_doc = match coll.find_one_and_replace(_filter, instance_doc, Some(opts))? {
            Some(doc) => doc,
            None => return Err(ResponseError("Server failed to return the updated document. Update may have failed.".to_owned())),
        };

        // Update instance ID if needed.
        if id_needs_update {
            let response_id = match updated_doc.get_object_id("_id") {
                Ok(id) => id,
                Err(_) => return Err(ResponseError("Server failed to return ObjectId of updated document.".to_owned())),
            };
            self.set_id(response_id.clone());
        };

        return Ok(());
    }

    /// Update the current model instance.
    ///
    /// As this method is simply a wrapper around MongoDB's
    /// [FindOneAndUpdate](https://docs.mongodb.com/v3.2/reference/method/db.collection.findOneAndUpdate/)
    /// operation, the `update` argument must be a valid update document. This operation targets the model,
    /// instance by the instance's ID. If its ID is `None`, this method will return an error.
    /// All other aspects of this method's input are passthrough.
    ///
    /// This method will consume `self`, and will return a new instance of `Self` based on the given
    /// return options (`ReturnDocument::Before | ReturnDocument:: After`).
    ///
    /// In order to provide consistent behavior, this method will also ensure that the operation's write
    /// concern `journaling` is set to `true`, so that we can receive a complete output document.
    ///
    /// If this model instance was never written to the database, this operation will return an error.
    fn update(self, db: Database, update: Document, opts: Option<FindOneAndUpdateOptions>) -> Result<Self> {
        let coll = db.collection(Self::COLLECTION_NAME);

        // Extract model's ID & use as filter for this operation.
        let id = match self.id() {
            Some(id) => id,
            None => {
                return Err(ArgumentError("Model must have an ObjectId for this operation.".to_owned()));
            }
        };
        let filter = doc!{"_id" => id};

        // Ensure that journaling is set to true for this call for full output document.
        // TODO: should probably encapsulate this as a unit-testable function.
        let options = match opts {
            Some(mut options) => {
                options.write_concern = match options.write_concern {
                    Some(mut wc) => {
                        wc.j = true;
                        Some(wc)
                    },
                    None => {
                        let mut wc = Self::model_write_concern();
                        wc.j = true;
                        Some(wc)
                    }
                };
                options
            },
            None => {
                let mut options = FindOneAndUpdateOptions::default();
                let mut wc = Self::model_write_concern();
                wc.j = true;
                options.write_concern = Some(wc);
                options
            }
        };

        // Perform a FindOneAndUpdate operation on this model's document by ID. Will fail if this
        // model instance was never saved to the database to begin with.
        let updated_doc = match coll.find_one_and_update(filter, update, Some(options))? {
            Some(doc) => doc,
            None => return Err(ResponseError("Server failed to return the updated document. Update may have failed.".to_owned())),
        };

        // Deserialize the return document into a model instance & return.
        return Self::instance_from_document(updated_doc);
    }

    /////////////////////////
    // Convenience Methods //

    /// Attempt to serialize the given bson document into an instance of this model.
    fn instance_from_document(document: bson::Document) -> Result<Self> {
        match bson::from_bson::<Self>(bson::Bson::Document(document)) {
            Ok(inst) => Ok(inst),
            Err(err) => Err(DecoderError(err)),
        }
    }

    ///////////////////////
    // Maintenance Layer //

    /// Get the vector of index models for this model.
    fn indexes() -> Vec<IndexModel> {
        vec![]
    }

    fn migrations() -> Vec<Box<Migration>> {
        vec![]
    }

    /// Synchronize this model with the backend.
    ///
    /// This routine should be called once per model, early on at boot time. It will synchronize
    /// any indexes defined on this model with the backend & will execute any active migrations
    /// against the model's collection.
    ///
    /// This routine will destroy any indexes found on this model's collection which are not
    /// defined in the response from `Self.indexes()`.
    fn sync(db: Database) -> Result<()> {
        let coll = db.collection(Self::COLLECTION_NAME);
        sync_model_indexes(&coll, Self::indexes())?;
        sync_model_migrations(&coll, Self::migrations())?;
        Ok(())
    }
}

fn sync_model_indexes<'a>(coll: &'a Collection, indexes: Vec<IndexModel>) -> Result<()> {
    info!("Synchronizing indexes for '{}'.", coll.namespace);

    // Fetch current indexes.
    let mut current_indexes_map: HashMap<String, Document> = HashMap::new();
    let indices = coll.list_indexes()
        .map_err(|err| DefaultError(format!("Error while fetching current indexes for '{}': {:?}", coll.namespace, err.description())))?
        .filter_map(|doc_res| doc_res.ok());
    for doc in indices {
        let idx_keys = doc.get_document("key")
            .map_err(|err| DefaultError(format!("Error extracting 'key' of index document: {:?}", err.description())))?;
        let key = idx_keys.keys().fold(String::from(""), |acc, bkey| acc + bkey);
        current_indexes_map.insert(key, doc.clone());
    }

    // Fetch target indexes for this model.
    let mut target_indexes_map: HashMap<String, IndexModel> = HashMap::new();
    for model in indexes.iter() {
        // Populate the 'target' indexes map for easy comparison later.
        let key = model.keys.keys().fold(String::from(""), |acc, bkey| acc + bkey);
        target_indexes_map.insert(key, model.to_owned());
    }

    // Determine which indexes must be created on the collection.
    let mut indexes_to_create = vec![];
    for (key, index_model) in target_indexes_map.iter() {
        // Check if key already exists.
        if !current_indexes_map.contains_key(key) {
            indexes_to_create.push(index_model)
        }
    }

    // Determine which indexes to remove.
    let mut indexes_to_remove = vec![];
    for (key, index_doc) in current_indexes_map {
        // Don't attempt to remove the default index.
        if &key == DEFAULT_INDEX {
            continue
        }

        // Check if key is not present in target indexes map. This means the index needs removal.
        if !target_indexes_map.contains_key(&key) {
            indexes_to_remove.push(index_doc);
        }
    }

    // Create needed indexes.
    for model in indexes_to_create {
        // NOTE: this wraps the native MongoDB `ensureIndex` command. Will not fail if index already exists.
        coll.create_index_model(model.clone())
            .map_err(|err| DefaultError(format!("Failed to create index: {}", err.description())))?;
    }

    // Remove old indexes.
    for doc in indexes_to_remove {
        let index_name = String::from(
            doc.get_str("name").map_err(|err| DefaultError(format!("Failed to get index name: {:?}", err.description())))?
        );
        coll.drop_index_string(index_name)
            .map_err(|err| DefaultError(format!("Failed to remove index: {}", err.description())))?;
    }

    info!("Finished synchronizing indexes for '{}'.", coll.namespace);
    Ok(())
}

fn sync_model_migrations<'a>(coll: &'a Collection, migrations: Vec<Box<Migration>>) -> Result<()> {
    info!("Starting migrations for '{}'.", coll.namespace);

    // Execute each migration.
    for migration in migrations {
        migration.execute(coll)?;
    }

    info!("Finished migrations for '{}'.", coll.namespace);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_index_options_returns_expected_output() {
        let output = basic_index_options("testing", true, None, None, None);

        assert!(output.name == Some("testing".to_string()));
        assert!(output.background == Some(true));
        assert!(output.unique == None);
        assert!(output.expire_after_seconds == None);
        assert!(output.sparse == None);
        assert!(output.storage_engine == None);
        assert!(output.version == None);
        assert!(output.default_language == None);
        assert!(output.language_override == None);
        assert!(output.text_version == None);
        assert!(output.weights == None);
        assert!(output.sphere_version == None);
        assert!(output.bits == None);
        assert!(output.max == None);
        assert!(output.min == None);
        assert!(output.bucket_size == None);
    }
}
