use chrono::{self, TimeZone};
use mongodb::{coll::options::IndexModel, oid::ObjectId, Document};
use wither::{self, prelude::*};

pub mod fixture;

pub use self::fixture::Fixture;

//////////////////////////////////////////////////////////////////////////////
// User //////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl<'a> Model<'a> for User {
    const COLLECTION_NAME: &'static str = "users";

    fn id(&self) -> Option<ObjectId> {
        return self.id.clone();
    }

    fn set_id(&mut self, oid: ObjectId) {
        self.id = Some(oid);
    }

    fn indexes() -> Vec<IndexModel> {
        return vec![IndexModel {
            keys: doc! {"email" => 1},
            options: wither::basic_index_options("unique-email", true, Some(true), None, None),
        }];
    }
}

impl<'m> Migrating<'m> for User {
    fn migrations() -> Vec<Box<wither::Migration>> {
        vec![
            // This migration doesn't really do much. Just exercises the system.
            Box::new(wither::IntervalMigration {
                name: String::from("test-migration"),
                threshold: chrono::Utc.ymd(2100, 1, 1).and_hms(1, 0, 0),
                filter: doc! {"email": doc!{"$exists": true}},
                set: Some(doc! {"testfield": "test"}),
                unset: None,
            }),
        ]
    }
}

//////////////////////////////////////////////////////////////////////////////
// UserModelBadMigrations ////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserModelBadMigrations {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl<'a> Model<'a> for UserModelBadMigrations {
    const COLLECTION_NAME: &'static str = "users_bad_migrations";

    fn id(&self) -> Option<ObjectId> {
        return self.id.clone();
    }

    fn set_id(&mut self, oid: ObjectId) {
        self.id = Some(oid);
    }

    fn indexes() -> Vec<IndexModel> {
        return vec![IndexModel {
            keys: doc! {"email" => 1},
            options: wither::basic_index_options("unique-email", true, Some(true), None, None),
        }];
    }
}

impl<'m> Migrating<'m> for UserModelBadMigrations {
    fn migrations() -> Vec<Box<wither::Migration>> {
        vec![
            // This migration doesn't really do much. Just exercises the system.
            Box::new(wither::IntervalMigration {
                name: String::from("test-migration"),
                threshold: chrono::Utc.ymd(2100, 1, 1).and_hms(1, 0, 0),
                filter: doc! {"email": doc!{"$exists": true}},
                set: None,
                unset: None,
            }),
        ]
    }
}

//////////////////////////////////////////////////////////////////////////////
// Derived Model /////////////////////////////////////////////////////////////

/// This model tests all of the major code generation bits.
#[derive(Serialize, Deserialize, Model, Default)]
#[model(collection_name = "derivations")]
pub struct DerivedModel {
    /// The ID of the model.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    // A field to test base line index options with index of type `asc`.
    #[model(index(
        index = "asc",
        name = "idx2",
        background = "true",
        sparse = "true",
        unique = "true",
        expire_after_seconds = "15",
        version = "1",
    ))]
    pub field0: String,

    // A field to test base line index options with index of type `dsc`.
    #[model(index(
        index = "dsc",
        name = "idx3",
        background = "false",
        sparse = "false",
        unique = "false",
        with(field = "text_field_a", index = "dsc"),
        with(field = "field0", index = "asc"),
    ))]
    pub field1: String,

    // A field to test index of type `text`.
    #[model(index(
        index = "text",
        name = "idx4",
        with(field = "text_field_b", index = "text"),
        weight(field = "text_field_a", weight = "10"),
        weight(field = "text_field_b", weight = "5"),
        text_version = "3",
        default_language = "en",
        language_override = "override_field",
    ))]
    pub text_field_a: String,
    pub text_field_b: String,

    // A field to test index of type `hashed`.
    #[model(index(index = "hashed", name = "idx5"))]
    pub hashed_field: String,
}

#[derive(Serialize, Deserialize, Model, Default)]
pub struct Derived2dModel {
    /// The ID of the model.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    // A field to test index of type `2d`.
    #[model(index(
        index = "2d",
        with(field = "field_2d_filter", index = "asc"),
        min = "-180.0",
        max = "180.0",
        bits = "1"
    ))]
    pub field_2d_a: Vec<f64>,
    pub field_2d_filter: String,
}

#[derive(Serialize, Deserialize, Model, Default)]
pub struct Derived2dsphereModel {
    /// The ID of the model.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    // A field to test index of type `2dsphere`.
    #[model(index(
        index = "2dsphere",
        sphere_version = "3",
        with(field = "field_2dsphere_filter", index = "asc")
    ))]
    pub field_2dsphere: Document,
    pub field_2dsphere_filter: String,
}

#[derive(Serialize, Deserialize, Model, Default)]
pub struct DerivedGeoHaystackModel {
    /// The ID of the model.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    // A field to test index of type `geoHaystack`.
    #[model(index(
        index = "geoHaystack",
        bucket_size = "5",
        with(field = "field_geo_haystack_filter", index = "asc")
    ))]
    pub field_geo_haystack: Document,
    pub field_geo_haystack_filter: String,
}
