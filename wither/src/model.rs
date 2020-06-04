//! Model related code.

use std::sync::Arc;

use bson::{doc, to_bson, from_bson};
use bson::{Bson, Document};
use bson::oid::ObjectId;

use mongodb::{Collection, Database};
use mongodb::options;
use mongodb::error::{Error, Result};
use mongodb::error::ErrorKind::{self, ArgumentError, ResponseError, BsonEncode};
use mongodb::results::DeleteResult;
use serde::{Serialize, de::DeserializeOwned};

use crate::cursor::ModelCursor;

/// The name of the default index created by MongoDB.
pub const DEFAULT_INDEX: &str = "_id";

/// This trait provides data modeling behaviors for interacting with MongoDB database collections.
///
/// Wither models are a thin abstraction over a standard MongoDB collection. Typically, the value
/// to be derived from using a model-based approach to working with your data will come about when
/// reading from and writing to the model's collection. For everything else, simply call the
/// `collection` method for direct access to the model's underlying collection handle.
///
#[cfg_attr(feature="docinclude", doc(include="../docs/model-derive.md"))]
#[cfg_attr(feature="docinclude", doc(include="../docs/model-sync.md"))]
#[cfg_attr(feature="docinclude", doc(include="../docs/logging.md"))]
#[cfg_attr(feature="docinclude", doc(include="../docs/manually-implementing-model.md"))]
#[cfg_attr(feature="docinclude", doc(include="../docs/underlying-driver.md"))]
pub trait Model where Self: Serialize + DeserializeOwned {

    /// The name of the collection where this model's data is stored.
    const COLLECTION_NAME: &'static str;

    /// Get the ID for this model instance.
    fn id(&self) -> Option<ObjectId>;

    /// Set the ID for this model.
    fn set_id(&mut self, id: ObjectId);

    //////////////////////////////////////////////////////////////////////////////////////////////
    // ReadConcern, WriteConcern & SelectionCritieria ////////////////////////////////////////////

    /// The model's read concern.
    fn read_concern() -> Option<options::ReadConcern> {
        None
    }

    /// The model's write concern.
    fn write_concern() -> Option<options::WriteConcern> {
        None
    }

    /// The model's selection criteria.
    ///
    /// When deriving a model, a function or an associated function should be specified which
    /// should be used to produce the desired value.
    fn selection_criteria() -> Option<options::SelectionCriteria> {
        None
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
    // Static Layer //////////////////////////////////////////////////////////////////////////////

    /// Get a handle to this model's collection.
    ///
    /// If there are any methods available on the underlying driver's collection object which are
    /// not available on the model interface, this is how you should access them. Typically,
    /// only methods which would be modified to deal with a model instance are actually wrapped
    /// by the model interface. Everything else should be accessed via this collection method.
    ///
    /// This method uses the model's `selection_criteria`, `read_concern` & `write_concern` when
    /// constructing the collection handle.
    fn collection(db: Database) -> Collection {
        db.collection_with_options(Self::COLLECTION_NAME, options::CollectionOptions{
            selection_criteria: Self::selection_criteria(),
            read_concern: Self::read_concern(),
            write_concern: Self::write_concern(),
        })
    }

    /// Find all instances of this model matching the given query.
    fn find<F, O>(db: Database, filter: F, options: O) -> Result<ModelCursor<Self>>
        where
            F: Into<Option<Document>>,
            O: Into<Option<options::FindOptions>>,
    {
        Ok(Self::collection(db)
            .find(filter, options)
            .map(ModelCursor::new)?)
    }

    /// Find the one model record matching your query, returning a model instance.
    fn find_one<F, O>(db: Database, filter: F, options: O) -> Result<Option<Self>>
        where
            F: Into<Option<Document>>,
            O: Into<Option<options::FindOneOptions>>,
    {
        Ok(Self::collection(db)
            .find_one(filter, options)?
            .map(|doc| Self::instance_from_document(doc))
            .transpose()?)
    }

    /// Finds a single document and deletes it, returning the original.
    fn find_one_and_delete<O>(db: Database, filter: Document, options: O) -> Result<Option<Self>>
        where O: Into<Option<options::FindOneAndDeleteOptions>>,
    {
        Ok(Self::collection(db).find_one_and_delete(filter, options)?
            .map(Self::instance_from_document)
            .transpose()?)
    }

    /// Finds a single document and replaces it, returning either the original or replaced document.
    fn find_one_and_replace<O>(db: Database, filter: Document, replacement: Document, options: O) -> Result<Option<Self>>
        where O: Into<Option<options::FindOneAndReplaceOptions>>,
    {
        Ok(Self::collection(db).find_one_and_replace(filter, replacement, options)?
            .map(Self::instance_from_document)
            .transpose()?)
    }

    /// Finds a single document and updates it, returning either the original or updated document.
    fn find_one_and_update<U, O>(db: Database, filter: Document, update: U, options: O) -> Result<Option<Self>>
        where
            U: Into<options::UpdateModifications>,
            O: Into<Option<options::FindOneAndUpdateOptions>>,
    {
        Ok(Self::collection(db).find_one_and_update(filter, update, options)?
            .map(Self::instance_from_document)
            .transpose()?)
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
    // Instance Layer ////////////////////////////////////////////////////////////////////////////

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
    ///
    /// **NOTE WELL:** in order to ensure needed behavior of this method, it will force `journaled`
    /// write concern.
    fn save(&mut self, db: Database, filter: Option<Document>) -> Result<()> {
        let coll = Self::collection(db);
        let instance_doc = Self::document_from_instance(&self)?;

        // Ensure that journaling is set to true for this call, as we need to be able to get an ID back.
        let mut write_concern = Self::write_concern().unwrap_or_default();
        write_concern.journal = Some(true);

        // Handle case where instance already has an ID.
        let mut id_needs_update = false;
        let filter = match (self.id(), filter) {
            (Some(id), _) => doc!{"_id": id},
            (None, None) => {
                let new_id = ObjectId::new().map_err(|err| Error::from(ErrorKind::ArgumentError{message: err.to_string()}))?;
                self.set_id(new_id.clone());
                doc!{"_id": new_id}
            }
            (None, Some(filter)) => {
                id_needs_update = true;
                filter
            }
        };

        // Save the record by replacing it entirely, or upserting if it doesn't already exist.
        let opts = options::FindOneAndReplaceOptions{
            upsert: Some(true),
            write_concern: Some(write_concern),
            return_document: Some(options::ReturnDocument::After),
            ..Default::default()
        };
        let updated_doc = coll.find_one_and_replace(filter, instance_doc, Some(opts))?
            .ok_or_else(|| ResponseError{message: "Server failed to return the updated document. Update may have failed.".into()})?;

        // Update instance ID if needed.
        if id_needs_update {
            let response_id = updated_doc.get_object_id("_id").map_err(|_| {
                ResponseError{message: "Server failed to return ObjectId of updated document.".into()}
            })?;
            self.set_id(response_id.clone());
        };
        Ok(())
    }

    /// Update the current model instance.
    ///
    /// This operation will always target the model instance by the instance's ID. If its ID is
    /// `None`, this method will return an error. If a filter document is provided, this method
    /// will ensure that the key `_id` is set to this model's ID.
    ///
    /// This method will consume `self`, and will return a new instance of `Self` based on the given
    /// return options (`ReturnDocument::Before | ReturnDocument:: After`).
    ///
    /// In order to provide consistent behavior, this method will also ensure that the operation's write
    /// concern `journaling` is set to `true`, so that we can receive a complete output document.
    ///
    /// If this model instance was never written to the database, this operation will return an error.
    fn update(self, db: Database, filter: Option<Document>, update: Document, opts: Option<options::FindOneAndUpdateOptions>) -> Result<Self> {
        // Extract model's ID & use as filter for this operation.
        let id = self.id().ok_or_else(|| ArgumentError{message: "Model must have an ObjectId for this operation.".into()})?;

        // Ensure we have a valid filter.
        let filter = match filter {
            Some(mut doc) => {
                doc.insert("_id", id);
                doc
            }
            None => doc!{"_id": id},
        };

        // Ensure that journaling is set to true for this call for full output document.
        let options = match opts {
            Some(mut options) => {
                options.write_concern = match options.write_concern {
                    Some(mut wc) => {
                        wc.journal = Some(true);
                        Some(wc)
                    },
                    None => {
                        let mut wc = Self::write_concern().unwrap_or_default();
                        wc.journal = Some(true);
                        Some(wc)
                    }
                };
                options
            },
            None => {
                let mut options = options::FindOneAndUpdateOptions::default();
                let mut wc = Self::write_concern().unwrap_or_default();
                wc.journal = Some(true);
                options.write_concern = Some(wc);
                options
            }
        };

        // Perform a FindOneAndUpdate operation on this model's document by ID.
        Ok(Self::collection(db).find_one_and_update(filter, update, Some(options))?
            .map(Self::instance_from_document)
            .transpose()?
            .ok_or_else(|| ResponseError{message: "Expected server to return a response document, none found.".into()})?)
    }

    /// Delete this model instance by ID.
    ///
    /// Wraps the driver's `Collection.delete_one` method.
    fn delete(&self, db: Database) -> Result<DeleteResult> {
        // Return an error if the instance was never saved.
        let id = self.id().ok_or_else(|| BsonEncode(bson::EncoderError::Unknown("This instance has no ID. It can not be deleted.".into())))?;
        Self::collection(db).delete_one(doc!{"_id": id}, None)
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
    // Convenience Methods ///////////////////////////////////////////////////////////////////////

    /// Attempt to serialize the given bson document into an instance of this model.
    fn instance_from_document(document: Document) -> Result<Self> {
        Ok(from_bson::<Self>(Bson::Document(document))?)
    }

    /// Attempt to serialize an instance of this model into a bson document.
    fn document_from_instance(&self) -> Result<Document> {
        to_bson(&self)
            .and_then(|val| match val {
                Bson::Document(doc) => Ok(doc),
                bsn @ _ => Err(bson::EncoderError::Unknown(format!("Expected Bson::Document found {:?}", bsn)))
            })
            .map_err(|err| Error{kind: Arc::new(BsonEncode(err))})
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
    // Maintenance Layer /////////////////////////////////////////////////////////////////////////

    /// Get the vector of index models for this model.
    fn indexes() -> Vec<options::IndexModel> {
        vec![]
    }

    /// Synchronize this model with the backend.
    ///
    /// This routine should be called once per model, early on at boottime. It will synchronize
    /// any indexes defined on this model with the backend.
    ///
    /// This routine will destroy any indexes found on this model's collection which are not
    /// defined in the response from `Self.indexes()`.
    fn sync(db: Database) -> Result<()> {
        let _coll = Self::collection(db);
        // let coll = Self::collection(db);
        // sync_model_indexes(&coll, Self::indexes())?; // TODO: blocked by https://jira.mongodb.org/projects/RUST/issues/RUST-166
        Ok(())
    }
}

// fn sync_model_indexes(coll: &Collection, indexes: Vec<options::IndexModel>) -> Result<()> {
//     let ns = coll.namespace();
//     log::info!("Synchronizing indexes for '{}'.", ns);

//     // Fetch current indexes.
//     let mut current_indexes_map: HashMap<String, Document> = HashMap::new();
//     let indices = coll.list_indexes()
//         .map_err(|err| DefaultError(format!("Error while fetching current indexes for '{}': {:?}", ns, err.description())))?
//         .filter_map(|doc_res| doc_res.ok());
//     for doc in indices {
//         let idx_keys = doc.get_document("key")
//             .map_err(|err| DefaultError(format!("Error extracting 'key' of index document: {:?}", err.description())))?;
//         let key = idx_keys.keys().fold(String::from(""), |acc, bkey| acc + bkey);
//         current_indexes_map.insert(key, doc.clone());
//     }

//     // Fetch target indexes for this model.
//     let mut target_indexes_map: HashMap<String, IndexModel> = HashMap::new();
//     for model in indexes.iter() {
//         // Populate the 'target' indexes map for easy comparison later.
//         let key = model.keys.keys().fold(String::from(""), |acc, bkey| acc + bkey);
//         target_indexes_map.insert(key, model.to_owned());
//     }

//     // Determine which indexes must be created on the collection.
//     let mut indexes_to_create = vec![];
//     for (key, index_model) in target_indexes_map.iter() {
//         // Check if key already exists.
//         if !current_indexes_map.contains_key(key) {
//             indexes_to_create.push(index_model)
//         }
//     }

//     // Determine which indexes to remove.
//     let mut indexes_to_remove = vec![];
//     for (key, index_doc) in current_indexes_map {
//         // Don't attempt to remove the default index.
//         if &key == DEFAULT_INDEX {
//             continue
//         }

//         // Check if key is not present in target indexes map. This means the index needs removal.
//         if !target_indexes_map.contains_key(&key) {
//             indexes_to_remove.push(index_doc);
//         }
//     }

//     // Create needed indexes.
//     for model in indexes_to_create {
//         // NOTE: this wraps the native MongoDB `ensureIndex` command. Will not fail if index already exists.
//         coll.create_index_model(model.clone())
//             .map_err(|err| DefaultError(format!("Failed to create index: {}", err)))?;
//     }

//     // Remove old indexes.
//     for doc in indexes_to_remove {
//         let index_name = String::from(
//             doc.get_str("name").map_err(|err| DefaultError(format!("Failed to get index name: {}", err)))?
//         );
//         coll.drop_index_string(index_name)
//             .map_err(|err| DefaultError(format!("Failed to remove index: {}", err)))?;
//     }

//     log::info!("Finished synchronizing indexes for '{}'.", ns);
//     Ok(())
// }
