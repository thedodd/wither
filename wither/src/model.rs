//! Model related code.

use std::collections::HashMap;

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

const MONGO_ID_INDEX_NAME: &str = "_id_";
const MONGO_DIFF_INDEX_BLACKLIST: [&str; 3] = ["v", "ns", "key"];

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
            .ok_or(WitherError::ServerFailedToReturnUpdatedDoc)?;

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
        let id = self.id().ok_or(WitherError::ModelIdRequiredForOperation)?;

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
            .ok_or(WitherError::ServerFailedToReturnUpdatedDoc)?)
    }

    /// Delete this model instance by ID.
    ///
    /// Wraps the driver's `Collection.delete_one` method.
    async fn delete(&self, db: &Database) -> Result<DeleteResult> {
        // Return an error if the instance was never saved.
        let id = self.id().ok_or(WitherError::ModelIdRequiredForOperation)?;
        Ok(Self::collection(db).delete_one(doc! {"_id": id}, None).await?)
    }

    /// Deletes all documents stored in the collection matching filter.
    ///
    /// Wraps the driver's `Collection.delete_many` method.
    async fn delete_many<O>(db: &Database, filter: Document, options: O) -> Result<DeleteResult>
    where
        O: Into<Option<options::DeleteOptions>> + Send,
    {
        Ok(Self::collection(db).delete_many(filter, options).await?)
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
    /// defined in this model's `indexes` method.
    async fn sync(db: &Database) -> Result<()> {
        let coll = Self::collection(db);
        let current_indexes = get_current_indexes(&db, &coll).await?;
        sync_model_indexes(db, &coll, Self::indexes(), current_indexes).await?;
        Ok(())
    }

    /// Get current collection indexes, if any.
    async fn get_current_indexes(db: &Database) -> Result<HashMap<String, IndexModel>> {
        let coll = Self::collection(db);
        get_current_indexes(db, &coll).await
    }
}

/// Get current collection indexes, if any.
async fn get_current_indexes(db: &Database, coll: &Collection) -> Result<HashMap<String, IndexModel>> {
    let list_indexes = match db.run_command(doc! {"listIndexes": coll.name()}, None).await {
        Ok(list_indexes) => list_indexes,
        Err(err) => match err.kind.as_ref() {
            // The DB & or collection does not yet exist. Move on.
            mongodb::error::ErrorKind::CommandError(err) if err.code == 26 => doc! {},
            _ => return Err(err.into()),
        },
    };
    Ok(build_index_map(list_indexes))
}

/// Generate an index name from the keys of the given document, matching the behavior of the
/// index management spec.
///
/// https://github.com/mongodb/specifications/blob/master/source/index-management.rst#index-name-generation
fn generate_index_name_from_keys(keys: &Document) -> String {
    let mut key = keys.iter().fold(String::from(""), |mut acc, (key, value)| {
        acc.push_str(&format!("{}_{}_", key, value.as_i32().unwrap_or(0)));
        acc
    });
    // Remove last underscore
    key.pop();
    key
}

/// Build a mapping of index names to their index models.
///
/// NOTE: this algorithm is sub-optimal and does not account for every possible error which may
/// arise during index processing. We would like to see the MongoDB team add the index management
/// commands back into their client, however this will do the trick for now. The only real concern
/// there is that the algorithm is not resilient to unexpected schema changes coming from the mongo
/// server. These changes are unlikely, but we are just documenting this fact here for posterity.
fn build_index_map(list_index: Document) -> HashMap<String, IndexModel> {
    // Unpack the cursor.
    let cursor = match list_index.get("cursor") {
        Some(cursor) => cursor,
        None => return Default::default(),
    };
    let doc = match cursor.as_document() {
        Some(doc) => doc,
        None => return Default::default(),
    };
    // https://docs.mongodb.com/manual/reference/limits/#Number-of-Indexes-per-Collection
    // We have a maximum of 64 indexes per collection, the firstBatch contains them all based on our tests.
    let first_batch = match doc.get_array("firstBatch").ok() {
        Some(first_batch) => first_batch,
        None => return Default::default(),
    };

    let index_map = first_batch
        .iter()
        // Extract documents.
        .filter_map(|bson| bson.as_document().cloned())
        // Filter out default index.
        .filter(|doc| {
            match doc.get_str("name").ok() {
                Some(name) if name == MONGO_ID_INDEX_NAME => false, // Filter out.
                Some(_) => true, // Include.
                None => false, // Filter out.
            }
        })
        .fold(HashMap::new(), |mut acc, doc| {
            // Extract document keys & generate index name based on keys.
            let idx_keys = match doc.get_document("key").ok() {
                Some(idx_keys) => idx_keys,
                None => return acc,
            };
            let index_name = generate_index_name_from_keys(idx_keys);

            // Build index model, filtering out blacklisted keys.
            let mut options = Document::new();
            doc.iter().for_each(|(b_key, b_value)| {
                if !MONGO_DIFF_INDEX_BLACKLIST.contains(&b_key.as_str()) {
                    options.insert(b_key.to_string(), b_value);
                }
            });
            let model = IndexModel::new(idx_keys.clone(), Some(options));

            acc.insert(index_name, model);
            acc
        });
    index_map
}

async fn sync_model_indexes<'a>(
    db: &'a Database, coll: &'a Collection, model_indexes: Vec<IndexModel>, current_indexes_map: HashMap<String, IndexModel>,
) -> Result<()> {
    log::info!("Synchronizing indexes for '{}'.", coll.namespace());

    // Build a mapping of aspired indexes based on the model's declared indexes.
    let aspired_indexes_map = model_indexes.iter().fold(HashMap::new(), |mut acc, model| {
        let mut target_model = model.clone();
        // Populate the 'target' indexes map for easy comparison later.
        let key = generate_index_name_from_keys(&model.keys);

        // Ensure we have an options object with at least the index name.
        match &mut target_model.options {
            Some(options) => {
                if options.get_str("name").ok().is_none() {
                    options.insert("name", key.clone());
                }
            }
            // If no options are present, then add a default options doc with the index name.
            None => {
                let options = doc! { "name": key.clone() };
                target_model.options = Some(options);
            }
        }
        acc.insert(key, target_model);
        acc
    });

    // For any current index which does not exist in the model's aspired indexes
    // list, add it to the drop list.
    let mut indexes_to_drop = current_indexes_map.iter().fold(vec![], |mut acc, (key, _)| {
        if !aspired_indexes_map.contains_key(key) {
            acc.push(key);
        }
        acc
    });

    // Diff aspired indexes with current indexes, and update our lists of indexes to create and
    // drop based on diffing the options of each index model. This is based purely on the
    // implementation of PartialEq on the bson::Document type.
    let mut indexes_to_create: HashMap<String, IndexModel> = HashMap::new();
    for (aspired_index_name, aspired_index) in aspired_indexes_map.iter() {
        // Unpack the corresponding current index by name if it exists, else prep it for creation.
        let current_index = match current_indexes_map.get(aspired_index_name) {
            Some(current_index) => current_index,
            // If the aspired index does not exist by name on the collection,
            // then we need to create it.
            None => {
                indexes_to_create.insert(aspired_index_name.clone(), aspired_index.clone());
                continue;
            }
        };

        // If the options of the two index models do not match, then we need to drop the existing
        // and create an updated version.
        if aspired_index.options != current_index.options {
            indexes_to_drop.push(aspired_index_name);
            indexes_to_create.insert(aspired_index_name.clone(), aspired_index.clone());
        }
    }

    // Drop indexes which have been flagged for dropping.
    for index_name in indexes_to_drop {
        let drop_command = doc! {
            "dropIndexes": coll.name(),
            "index": index_name,
        };
        db.run_command(drop_command, None).await?;
    }

    // Create any indexes which have been flagged for creation.
    let indexes_to_create = indexes_to_create.into_iter().fold(vec![], |mut acc, (_, index_model)| {
        let mut index_doc = Document::new();
        index_doc.insert("key", index_model.keys);
        if let Some(options) = index_model.options {
            index_doc.extend(options);
        }
        acc.push(index_doc);
        acc
    });
    if !indexes_to_create.is_empty() {
        db.run_command(
            doc! {
                "createIndexes": coll.name(),
                "indexes": indexes_to_create,
            },
            None,
        )
        .await?;
    }

    log::info!("Synchronized indexes for '{}'.", coll.namespace());

    Ok(())
}
