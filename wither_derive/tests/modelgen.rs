#[macro_use]
extern crate bson;
extern crate compiletest_rs as compiletest;
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;
#[macro_use(Model)]
extern crate wither_derive;

use bson::Document;
use wither::prelude::*;
use mongodb::coll::options::IndexModel;

/// This model tests all of the major code generation bits.
///
/// NOTE: do not attempt to sync indices for this model, as MongoDB will reject it for having
/// too many geo indexes. This is just for testing code generation.
#[derive(Serialize, Deserialize, Model, Default)]
#[model(collection_name="derivations")]
struct DerivedModel {
    /// The ID of the model.
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,

    // A field to test base line index options with index of type `asc`.
    #[model(index(
        index_type="asc",
        background="true", sparse="true", unique="true", expire_after_seconds="15", name="field0", version="1",
    ))]
    pub field0: String,

    // A field to test base line index options with index of type `dsc`.
    #[model(index(
        index_type="dsc",
        background="false", sparse="false", unique="false", with(text_field_a="dsc", field0="asc"),
    ))]
    pub field1: String,

    // A field to test index of type `text`.
    #[model(index(
        index_type="text", with(text_field_b="text"), weights(text_field_a="10", text_field_b="5"),
        text_version="3", default_language="en", language_override="override_field",
    ))]
    pub text_field_a: String,
    pub text_field_b: String,

    // A field to test index of type `hashed`.
    #[model(index(index_type="hashed"))]
    pub hashed_field: String,

    // A field to test index of type `2d`.
    #[model(index(index_type="2d", with(field_2d_b="2d"), min="-180.0", max="180.0", bits="1"))]
    pub field_2d_a: Vec<f64>,
    pub field_2d_b: Vec<f64>,

    // A field to test index of type `2dsphere`.
    #[model(index(index_type="2dsphere", sphere_version="3", with(field_2dsphere_filter="asc")))]
    pub field_2dsphere: Document,
    pub field_2dsphere_filter: String,

    // A field to test index of type `geoHaystack`.
    #[model(index(index_type="geoHaystack", bucket_size="5", with(field_geo_haystack_filter="asc")))]
    pub field_geo_haystack: Document,
    pub field_geo_haystack_filter: String,
}

/// This model tests the generation of an accurate collection name based on the model's ident.
#[derive(Model, Serialize, Deserialize)]
struct SecondModel {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    id: Option<bson::oid::ObjectId>,
}

#[test]
fn test_first_model_collection_name() {
    assert_eq!("derivations", DerivedModel::COLLECTION_NAME);
}

#[test]
fn test_second_model_collection_name() {
    assert_eq!("second_models", SecondModel::COLLECTION_NAME);
}

#[test]
fn test_first_model_indexes() {
    let indexes = DerivedModel::indexes();
    assert_eq!(7, indexes.len());

    let (idx1, idx2, idx3, idx4, idx5, idx6, idx7) = (
        indexes[0].clone(), indexes[1].clone(), indexes[2].clone(), indexes[3].clone(),
        indexes[4].clone(), indexes[5].clone(), indexes[6].clone()
    );

    IndexShapeAssert::new(idx1, doc!{"field0": 1i32})
        .background(Some(true)).sparse(Some(true)).unique(Some(true)).version(Some(1i32))
        .expire_after_seconds(Some(15i32)).name(Some("field0".to_string()))
        .assert();

    IndexShapeAssert::new(idx2, doc!{"field1": -1i32, "text_field_a": -1i32, "field0": 1i32})
        .background(Some(false)).sparse(Some(false)).unique(Some(false)).assert();

    IndexShapeAssert::new(idx3, doc!{"text_field_a": String::from("text"), "text_field_b": String::from("text")})
        .default_language(Some("en".to_string()))
        .language_override(Some("override_field".to_string()))
        .text_version(Some(3i32))
        .weights(Some(doc!{"text_field_a": 10i32, "text_field_b": 5i32}))
        .assert();

    IndexShapeAssert::new(idx4, doc!{"hashed_field": "hashed"}).assert();

    IndexShapeAssert::new(idx5, doc!{"field_2d_a": "2d", "field_2d_b": "2d"})
        .min(Some(-180.0f64)).max(Some(180.0f64)).bits(Some(1i32)).assert();

    IndexShapeAssert::new(idx6, doc!{"field_2dsphere": "2dsphere", "field_2dsphere_filter": 1i32})
        .sphere_version(Some(3i32)).assert();

    IndexShapeAssert::new(idx7, doc!{"field_geo_haystack": "geoHaystack", "field_geo_haystack_filter": 1i32})
        .bucket_size(Some(5i32)).assert();
}

/// A utility type used for comparing index models.
///
/// TODO: get rid of this once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/288 lands.
struct IndexShapeAssert {
    model: IndexModel,
    keys: Document,
    background: Option<bool>,
    expire_after_seconds: Option<i32>,
    name: Option<String>,
    sparse: Option<bool>,
    storage_engine: Option<String>,
    unique: Option<bool>,
    version: Option<i32>,
    default_language: Option<String>,
    language_override: Option<String>,
    text_version: Option<i32>,
    weights: Option<Document>,
    sphere_version: Option<i32>,
    bits: Option<i32>,
    max: Option<f64>,
    min: Option<f64>,
    bucket_size: Option<i32>,
}

impl IndexShapeAssert {
    /// Build a new instance for asserting on index shape.
    pub fn new(model: IndexModel, keys: Document) -> Self {
        IndexShapeAssert{
            model, keys,
            background: None, expire_after_seconds: None, name: None, sparse: None, storage_engine: None,
            unique: None, version: None, default_language: None, language_override: None, text_version: None,
            weights: None, sphere_version: None, bits: None, max: None, min: None, bucket_size: None,
        }
    }

    /// Execute a comparrison against the index `model`.
    pub fn assert(self) {
        assert_eq!(self.model.keys, self.keys);
        assert_eq!(self.model.options.background, self.background);
        assert_eq!(self.model.options.bits, self.bits);
        assert_eq!(self.model.options.bucket_size, self.bucket_size);
        assert_eq!(self.model.options.default_language, self.default_language);
        assert_eq!(self.model.options.expire_after_seconds, self.expire_after_seconds);
        assert_eq!(self.model.options.language_override, self.language_override);
        assert_eq!(self.model.options.max, self.max);
        assert_eq!(self.model.options.min, self.min);
        assert_eq!(self.model.options.name, self.name);
        assert_eq!(self.model.options.sparse, self.sparse);
        assert_eq!(self.model.options.sphere_version, self.sphere_version);
        assert_eq!(self.model.options.storage_engine, self.storage_engine);
        assert_eq!(self.model.options.text_version, self.text_version);
        assert_eq!(self.model.options.unique, self.unique);
        assert_eq!(self.model.options.version, self.version);
        assert_eq!(self.model.options.weights, self.weights);
    }

    pub fn background(mut self, background: Option<bool>) -> Self {
        self.background = background;
        self
    }

    pub fn expire_after_seconds(mut self, expire_after_seconds: Option<i32>) -> Self {
        self.expire_after_seconds = expire_after_seconds;
        self
    }

    pub fn name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn sparse(mut self, sparse: Option<bool>) -> Self {
        self.sparse = sparse;
        self
    }

    #[allow(dead_code)] // NOTE: we'll test this once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/282 lands.
    pub fn storage_engine(mut self, storage_engine: Option<String>) -> Self {
        self.storage_engine = storage_engine;
        self
    }

    pub fn unique(mut self, unique: Option<bool>) -> Self {
        self.unique = unique;
        self
    }

    pub fn version(mut self, version: Option<i32>) -> Self {
        self.version = version;
        self
    }

    pub fn default_language(mut self, default_language: Option<String>) -> Self {
        self.default_language = default_language;
        self
    }

    pub fn language_override(mut self, language_override: Option<String>) -> Self {
        self.language_override = language_override;
        self
    }

    pub fn text_version(mut self, text_version: Option<i32>) -> Self {
        self.text_version = text_version;
        self
    }

    pub fn weights(mut self, weights: Option<Document>) -> Self {
        self.weights = weights;
        self
    }

    pub fn sphere_version(mut self, sphere_version: Option<i32>) -> Self {
        self.sphere_version = sphere_version;
        self
    }

    pub fn bits(mut self, bits: Option<i32>) -> Self {
        self.bits = bits;
        self
    }

    pub fn max(mut self, max: Option<f64>) -> Self {
        self.max = max;
        self
    }

    pub fn min(mut self, min: Option<f64>) -> Self {
        self.min = min;
        self
    }

    pub fn bucket_size(mut self, bucket_size: Option<i32>) -> Self {
        self.bucket_size = bucket_size;
        self
    }
}
