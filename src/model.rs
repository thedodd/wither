use std::collections::HashMap;
use std::error::Error;

use bson;
use bson::Document;
use bson::oid::ObjectId;
use mongodb;
use mongodb::error::Error::{
    DecoderError,
    DefaultError,
    OIDError,
    ResponseError,
};
use mongodb::coll::options::{
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
///
/// This allows you to define a data model using a normal struct, and then interact with your
/// MongoDB database collections using that struct.
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

    /// Find all instances of this model matching the given query.
    fn find(db: Database, filter: Option<Document>, options: Option<FindOptions>) -> mongodb::error::Result<Vec<Self>> {
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

    /// Find the one model record matching your query, returning a model instance.
    fn find_one(db: Database, filter: Option<Document>, options: Option<FindOptions>) -> mongodb::error::Result<Option<Self>> {
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
    fn save(&mut self, db: Database, filter: Option<Document>) -> mongodb::error::Result<()> {
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

    /////////////////////////
    // Convenience Methods //

    /// Attempt to serialize the given bson document into an instance of this model.
    fn instance_from_document(document: bson::Document) -> mongodb::error::Result<Self> {
        match bson::from_bson::<Self>(bson::Bson::Document(document)) {
            Ok(inst) => Ok(inst),
            Err(err) => Err(DecoderError(err)),
        }
    }

    /////////////////
    // Index Layer //

    /// Get the vector of index models for this model.
    fn indexes() -> Vec<IndexModel> {
        return vec![];
    }

    /// Synchronize this model with the backend.
    ///
    /// This routine will destroy any indexes found on this model's collection which are not
    /// defined in the response from `Self.indexes()`.
    ///
    /// This routine should be called once per model, early on at boot time.
    ///
    /// TODO:
    /// - build up a safe sync execution standpoint.
    /// - return before doing anything if index sync can not be executed safely.
    fn sync(db: Database) {

        let coll = db.collection(Self::COLLECTION_NAME);
        println!("Synchronizing indexes for collection model: '{}'.", Self::COLLECTION_NAME); // TODO: logging: debug.

        // Fetch current indexes.
        let mut current_indexes_map: HashMap<String, Document> = HashMap::new();
        let err_msg = format!("Error while fetching current indexes for '{}'.", Self::COLLECTION_NAME);
        if let Ok(cursor) = coll.list_indexes() {
            for doc_opt in cursor {
                let doc = doc_opt.expect(&err_msg);
                let idx_keys = doc.get_document("key").expect("Returned index appears to be malformed.");
                let key = idx_keys.keys().fold("".to_owned(), |acc, bkey| acc + bkey);
                current_indexes_map.insert(key, doc.clone());
            }
        }

        // Fetch target indexes for this model.
        let mut target_indexes_map: HashMap<String, IndexModel> = HashMap::new();
        for model in Self::indexes().iter() {
            // Populate the 'target' indexes map for easy comparison later.
            let key = model.keys.keys().fold("".to_owned(), |acc, bkey| acc + bkey);
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
            println!("Syncing index: {:?}", model); // TODO: logging: debug.
            match coll.create_index_model(model.clone()) {
                Ok(_) => println!("Index synced: {:?}", model), // TODO: logging: debug.
                Err(err) => panic!("Failed to create index: {}", err.description()),
            };
        }

        // Remove old indexes.
        for doc in indexes_to_remove {
            println!("Removing index: {:?}", doc); // TODO: logging: debug.
            match coll.drop_index_string(doc.get_str("name").expect("Expected to find index name.").to_owned()) {
                Ok(_) => println!("Index removed: {:?}", doc), // TODO: logging: debug.
                Err(err) => panic!("Failed to remove index: {}", err.description()),
            };
        }
        println!("Finished synchronizing indexes."); // TODO: logging: debug.
    }
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
