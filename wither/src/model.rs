//! Model related code.

use async_trait::async_trait;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, from_bson, to_bson};
use mongodb::bson::{Bson, Document};

use mongodb::options;
use mongodb::results::DeleteResult;
use mongodb::{Collection, Database};
use serde::{de::DeserializeOwned, Serialize};

use crate::common::IndexModel;
use crate::cursor::ModelCursor;
use crate::error::{Result, WitherError};
use std::collections::HashMap;

/// This trait provides data modeling behaviors for interacting with MongoDB database collections.
///
/// Wither models are a thin abstraction over a standard MongoDB collection. Typically, the value
/// gained from using a model-based approach to working with your data will come about when
/// reading from and writing to the model's collection. For everything else, simply call the
/// `collection` method for direct access to the model's underlying collection handle.
///
/// Any `read_concern`, `write_concern` or `selection_criteria` options configured for the model,
/// either derived or manually, will be used for collection interactions.
#[cfg_attr(feature = "docinclude", doc(include = "../docs/model-derive.md"))]
#[cfg_attr(feature = "docinclude", doc(include = "../docs/model-sync.md"))]
#[cfg_attr(feature = "docinclude", doc(include = "../docs/logging.md"))]
#[cfg_attr(feature = "docinclude", doc(include = "../docs/underlying-driver.md"))]
#[async_trait]
pub trait Model
where
    Self: Serialize + DeserializeOwned,
{
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
    fn collection(db: &Database) -> Collection {
        db.collection_with_options(
            Self::COLLECTION_NAME,
            options::CollectionOptions::builder()
                .selection_criteria(Self::selection_criteria())
                .read_concern(Self::read_concern())
                .write_concern(Self::write_concern())
                .build(),
        )
    }

    /// Find all instances of this model matching the given query.
    async fn find<F, O>(db: &Database, filter: F, options: O) -> Result<ModelCursor<Self>>
    where
        F: Into<Option<Document>> + Send,
        O: Into<Option<options::FindOptions>> + Send,
    {
        Ok(Self::collection(db).find(filter, options).await.map(ModelCursor::new)?)
    }

    /// Find the one model record matching your query, returning a model instance.
    async fn find_one<F, O>(db: &Database, filter: F, options: O) -> Result<Option<Self>>
    where
        F: Into<Option<Document>> + Send,
        O: Into<Option<options::FindOneOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one(filter, options)
            .await?
            .map(Self::instance_from_document)
            .transpose()?)
    }

    /// Finds a single document and deletes it, returning the original.
    async fn find_one_and_delete<O>(db: &Database, filter: Document, options: O) -> Result<Option<Self>>
    where
        O: Into<Option<options::FindOneAndDeleteOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one_and_delete(filter, options)
            .await?
            .map(Self::instance_from_document)
            .transpose()?)
    }

    /// Finds a single document and replaces it, returning either the original or replaced document.
    async fn find_one_and_replace<O>(db: &Database, filter: Document, replacement: Document, options: O) -> Result<Option<Self>>
    where
        O: Into<Option<options::FindOneAndReplaceOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one_and_replace(filter, replacement, options)
            .await?
            .map(Self::instance_from_document)
            .transpose()?)
    }

    /// Finds a single document and updates it, returning either the original or updated document.
    async fn find_one_and_update<U, O>(db: &Database, filter: Document, update: U, options: O) -> Result<Option<Self>>
    where
        U: Into<options::UpdateModifications> + Send,
        O: Into<Option<options::FindOneAndUpdateOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one_and_update(filter, update, options)
            .await?
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
    async fn save(&mut self, db: &Database, filter: Option<Document>) -> Result<()> {
        let coll = Self::collection(db);
        let instance_doc = Self::document_from_instance(&self)?;

        // Ensure that journaling is set to true for this call, as we need to be able to get an ID back.
        let mut write_concern = Self::write_concern().unwrap_or_default();
        write_concern.journal = Some(true);

        // Handle case where instance already has an ID.
        let mut id_needs_update = false;
        let filter = match (self.id(), filter) {
            (Some(id), _) => doc! {"_id": id},
            (None, None) => {
                let new_id = ObjectId::new();
                self.set_id(new_id.clone());
                doc! {"_id": new_id}
            }
            (None, Some(filter)) => {
                id_needs_update = true;
                filter
            }
        };

        // Save the record by replacing it entirely, or upserting if it doesn't already exist.
        let opts = options::FindOneAndReplaceOptions::builder()
            .upsert(Some(true))
            .write_concern(Some(write_concern))
            .return_document(Some(options::ReturnDocument::After))
            .build();
        let updated_doc = coll
            .find_one_and_replace(filter, instance_doc, Some(opts))
            .await?
            .ok_or_else(|| WitherError::ServerFailedToReturnUpdatedDoc)?;

        // Update instance ID if needed.
        if id_needs_update {
            let response_id = updated_doc.get_object_id("_id").map_err(|_| WitherError::ServerFailedToReturnObjectId)?;
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
    /// In order to provide consistent behavior, this method will also ensure that the operation's
    /// write concern `journaling` is set to `true`, so that we can receive a complete output
    /// document.
    ///
    /// If this model instance was never written to the database, this operation will return an
    /// error.
    async fn update(self, db: &Database, filter: Option<Document>, update: Document, opts: Option<options::FindOneAndUpdateOptions>) -> Result<Self> {
        // Extract model's ID & use as filter for this operation.
        let id = self.id().ok_or_else(|| WitherError::ModelIdRequiredForOperation)?;

        // Ensure we have a valid filter.
        let filter = match filter {
            Some(mut doc) => {
                doc.insert("_id", id);
                doc
            }
            None => doc! {"_id": id},
        };

        // Ensure that journaling is set to true for this call for full output document.
        let options = match opts {
            Some(mut options) => {
                options.write_concern = match options.write_concern {
                    Some(mut wc) => {
                        wc.journal = Some(true);
                        Some(wc)
                    }
                    None => {
                        let mut wc = Self::write_concern().unwrap_or_default();
                        wc.journal = Some(true);
                        Some(wc)
                    }
                };
                options
            }
            None => {
                let mut options = options::FindOneAndUpdateOptions::default();
                let mut wc = Self::write_concern().unwrap_or_default();
                wc.journal = Some(true);
                options.write_concern = Some(wc);
                options
            }
        };

        // Perform a FindOneAndUpdate operation on this model's document by ID.
        Ok(Self::collection(db)
            .find_one_and_update(filter, update, Some(options))
            .await?
            .map(Self::instance_from_document)
            .transpose()?
            .ok_or_else(|| WitherError::ServerFailedToReturnUpdatedDoc)?)
    }

    /// Delete this model instance by ID.
    ///
    /// Wraps the driver's `Collection.delete_one` method.
    async fn delete(&self, db: &Database) -> Result<DeleteResult> {
        // Return an error if the instance was never saved.
        let id = self.id().ok_or_else(|| WitherError::ModelIdRequiredForOperation)?;
        Ok(Self::collection(db).delete_one(doc! {"_id": id}, None).await?)
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
    // Convenience Methods ///////////////////////////////////////////////////////////////////////

    /// Attempt to serialize the given bson document into an instance of this model.
    fn instance_from_document(document: Document) -> Result<Self> {
        Ok(from_bson::<Self>(Bson::Document(document))?)
    }

    /// Attempt to serialize an instance of this model into a bson document.
    fn document_from_instance(&self) -> Result<Document> {
        match to_bson(&self)? {
            Bson::Document(doc) => Ok(doc),
            bsn => Err(WitherError::ModelSerToDocument(bsn.element_type())),
        }
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
    // Maintenance Layer /////////////////////////////////////////////////////////////////////////

    /// Get the vector of index models for this model.
    fn indexes() -> Vec<IndexModel> {
        vec![]
    }

    /// Synchronize this model with the backend.
    ///
    /// This routine should be called once per model, early on at boottime. It will synchronize
    /// any indexes defined on this model with the backend.
    ///
    /// This routine will destroy any indexes found on this model's collection which are not
    /// defined in the response from `Self.indexes()`.
    async fn sync(db: &Database) -> Result<()> {
        let coll = db.collection(Self::COLLECTION_NAME);
        sync_model_indexes(db, &coll, Self::indexes()).await?;
        Ok(())
    }

    /// Get current collection indexes if any
    async fn get_current_indexes(db: &Database) -> HashMap<String, IndexModel> {
        // Get the existing indexes, this result is an error when there are no indexes
        let list_index_result = db.run_command(doc! {"listIndexes": Self::COLLECTION_NAME}, None).await;

        create_current_indexes_map(list_index_result)
    }
}

const MONGO_ID_INDEX_NAME: &str = "_id_";
const MONGO_DIFF_INDEX_BLACKLIST: [&str; 3] = ["v", "ns", "key"];

// Reference: https://github.com/mongodb/specifications/blob/master/source/index-management.rst#index-name-generation
fn generate_index_name_from_keys(keys: &Document) -> String {
    let mut key = keys.iter().fold(String::from(""), |mut acc, (key, value)| {
        acc.push_str(&format!("{}_{}_", key, value.as_i32().unwrap_or(0)));
        acc
    });
    // Remove last underscore
    key.pop();
    key
}

fn create_current_indexes_map(list_index_result: std::result::Result<Document, mongodb::error::Error>) -> HashMap<String, IndexModel> {
    let mut current_indexes_map: HashMap<String, IndexModel> = HashMap::new();

    if let Ok(doc) = list_index_result {
        // There are indexes
        if let Some(cursor) = doc.get("cursor") {
            let doc = cursor.as_document().unwrap();

            // https://docs.mongodb.com/manual/reference/limits/#Number-of-Indexes-per-Collection
            // We have a maximum of 64 indexes per collection, the firstBatch contains them all in my tests
            // If anyone is capable to understand how to make this batch overflow let me know and I'll fix it
            let first_batch = doc.get_array("firstBatch").unwrap();

            first_batch
                .iter()
                // Convert to document
                .map(|bson| bson.as_document().unwrap().clone())
                // Remove default index
                .filter(|doc| doc.get_str("name").unwrap() != MONGO_ID_INDEX_NAME)
                .for_each(|doc| {
                    // Index keys
                    let idx_keys = doc.get_document("key").unwrap();

                    // Unique generated key for the map
                    let key = generate_index_name_from_keys(idx_keys);

                    // These are the options
                    let mut options = Document::new();

                    doc.iter().for_each(|(b_key, b_value)| {
                        // Remove unused stuff (key, namespace, version)
                        let key_in_blacklist = !MONGO_DIFF_INDEX_BLACKLIST
                            .iter()
                            .all(|blacklisted| b_key != blacklisted);
                        if !key_in_blacklist {
                            options.insert(b_key, b_value);
                        }
                    });

                    // Custom IndexModel
                    let model = IndexModel::new(idx_keys.clone(), Some(options));

                    // Add the IndexModel to the current_indexes_map
                    current_indexes_map.insert(key, model);
                });
        }
    }

    current_indexes_map
}

async fn sync_model_indexes<'a>(db: &'a Database, coll: &'a Collection, model_indexes: Vec<IndexModel>) -> Result<()> {
    log::info!("Synchronizing indexes for '{}'.", coll.namespace());

    // Get the existing indexes, this result is an error when there are no indexes
    let list_index_result = db.run_command(doc! {"listIndexes": coll.name()}, None).await;

    // The already present indexes
    let current_indexes_map = create_current_indexes_map(list_index_result);

    // Fetch target indexes for this model.
    let mut target_indexes_map: HashMap<String, IndexModel> = HashMap::new();
    for model in model_indexes.iter() {
        let mut target_model = model.clone();
        // Populate the 'target' indexes map for easy comparison later.
        let key = generate_index_name_from_keys(&model.keys);

        // If we have options
        if let Some(ref mut options) = target_model.options {
            // Try to get the name
            if options.get_str("name").is_err() {
                // If the name option is missing add it
                options.insert("name", key.clone());
            }
        } else {
            // If the model has no option then add the name
            let options = doc! {"name": key.clone()};
            target_model.options = Some(options);
        }

        target_indexes_map.insert(key, target_model.clone());
    }

    // These are the indexes that must be dropped (I just need the name)
    let mut indexes_to_drop: Vec<String> = Vec::new();

    // These are the indexes that must be created
    let mut indexes_to_create: HashMap<String, IndexModel> = HashMap::new();

    // First drop the additional current_indexes
    current_indexes_map.iter().for_each(|(key, current_index)| {
        // There is no key in target
        if target_indexes_map.get(key).is_none() {
            indexes_to_drop.push(String::from(current_index.options.as_ref().unwrap().get_str("name").unwrap()));
        }
    });

    // Iter over target_indexes
    for (key, index_model) in target_indexes_map.iter() {
        // Check if key already exists
        if let Some(current_index) = current_indexes_map.get(key) {
            /*
            // First check differences in the key
            let current_index_keys = &current_index.keys;
            let target_index_keys = &index_model.keys;

            // If it exist we must do some checks
            let has_different_key =
            // There is a difference if any current keys is different from any target keys
            current_index_keys.iter().any(|(current_key, current_value)| {
                if let Some(target_value) = target_index_keys.get(current_key) {
                    // This is the order of the key (+1 or -1)
                    target_value != current_value
                } else {
                    // Added a key to the index keys
                    false
                }
            });
            // There is a difference, drop the index
            if has_different_key {
                indexes_to_drop.push(String::from(
                    current_index
                        .options
                        .as_ref()
                        .unwrap()
                        .get_str("name")
                        .unwrap(),
                ));
                indexes_to_create.insert(key.clone(), index_model.clone());
            } else {
            */
            // We need to check options
            // If the index is already present there must be the "name" option so "options" is always defined
            let current_index_options_doc = &current_index.options.as_ref().unwrap();
            // The target index could have no options
            let target_index_options_doc = &index_model.options.as_ref().unwrap();

            // if let Some(target_index_options_doc) = target_index_options {
            // Iter over target_index_options_doc
            let has_diff = target_index_options_doc.iter().any(|(target_key, target_value)| {
                // If the target option is also in current index option
                let current_index_option = current_index_options_doc.get(target_key);
                // We have a diff if ANY current option is not equal to the target option
                if let Some(current_value) = current_index_option {
                    current_value != target_value
                } else {
                    true
                }
            });
            if has_diff {
                indexes_to_drop.push(String::from(current_index_options_doc.get_str("name").unwrap()));
                indexes_to_create.insert(key.clone(), index_model.clone());
            }
        // } else {
        //     // This can be tricky, if the target_index has no options it could also mean that the user did not specify a name
        //     // But in the current_index there is always at least the name which is generated from the key
        //     // let has_custom_options = current_index_options_doc.iter().any(|(k, _)| {
        //     //     k != "name"
        //     // });
        //     // if has_custom_options {
        //     indexes_to_drop.push(String::from(
        //         current_index_options_doc.get_str("name").unwrap(),
        //     ));
        //     indexes_to_create.insert(key.clone(), index_model.clone());
        //     // }
        // }
        // }
        } else {
            // If it doesn't just add it
            indexes_to_create.insert(key.clone(), index_model.clone());
        }
    }

    // `To drop multiple indexes (Available starting in MongoDB 4.2), specify an array of the index
    // names` If we drop support for MongoDB <= 4.2 we could drop all indexes at once
    for drop_key in indexes_to_drop {
        let drop_command = doc! {
            "dropIndexes": coll.name(),
            "index": drop_key
        };

        db.run_command(drop_command, None).await?;
    }

    let mut create_command = doc! {
        "createIndexes": coll.name(),
    };

    let mut indexes_doc: Vec<Document> = Vec::new();

    // Foreach index prepare the documents
    for (_, model) in indexes_to_create.iter() {
        let mut options = Document::new();
        if let Some(opts) = model.options.clone() {
            options = opts;
        }

        let key = model.keys.clone();

        let mut doc_index = Document::new();

        doc_index.insert("key", key);
        doc_index.extend(options);

        indexes_doc.push(doc_index);
    }

    if !indexes_doc.is_empty() {
        create_command.insert("indexes", indexes_doc);
        db.run_command(create_command, None).await?;
    }

    log::info!("Synchronized indexes for '{}'.", coll.namespace());

    Ok(())
}
