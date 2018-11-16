extern crate chrono;
#[macro_use]
extern crate lazy_static;
#[macro_use(doc, bson)]
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;
#[macro_use(Model)]
extern crate wither_derive;

mod fixtures;

use std::error::Error;

use mongodb::{
    Document,
    coll::options::{FindOneAndUpdateOptions, ReturnDocument},
    db::ThreadedDatabase,
};
use wither::prelude::*;

use fixtures::{
    Fixture,
    User,
    UserModelBadMigrations,
    DerivedModel,
    Derived2dModel,
    Derived2dsphereModel,
    // DerivedGeoHaystackModel,
};

//////////////////////////////////////////////////////////////////////////////
// Model::count //////////////////////////////////////////////////////////////

#[test]
fn model_count_should_return_expected_count_matching_filter() {
    let fixture = Fixture::new().with_dropped_database().with_synced_models();
    let db = fixture.get_db();
    let mut user = User{id: None, email: "test@test.com".to_string()};
    user.save(db.clone(), None).expect("Expected a successful save operation.");
    let doc = doc!{"_id" => (user.id.clone().unwrap())};

    let count = User::count(db.clone(), Some(doc), None)
        .expect("Expected a successful count operation.");

    assert_eq!(count, 1);
}

//////////////////////////////////////////////////////////////////////////////
// Model.save ////////////////////////////////////////////////////////////////

#[test]
fn model_save_should_save_model_instance_and_add_id() {
    let fixture = Fixture::new().with_dropped_database().with_synced_models();
    let db = fixture.get_db();
    let mut user = User{id: None, email: "test@test.com".to_string()};

    user.save(db.clone(), None).expect("Expected a successful save operation.");

    assert!(user.id != None)
}

#[test]
fn derived_model_save_should_save_model_instance_and_add_id() {
    let fixture = Fixture::new().with_dropped_database().with_synced_models();
    let db = fixture.get_db();
    let mut model = DerivedModel::default();

    model.save(db.clone(), None).expect("Expected a successful save operation.");

    assert!(model.id != None)
}

//////////////////////////////////////////////////////////////////////////////
// Model::find ///////////////////////////////////////////////////////////////

#[test]
fn model_find_should_find_all_instances_of_model_with_no_filter_or_options() {
    let fixture = Fixture::new().with_dropped_database().with_synced_models();
    let db = fixture.get_db();
    let mut user = User{id: None, email: "test@test.com".to_string()};
    user.save(db.clone(), None).expect("Expected a successful save operation.");

    let users_from_db = User::find(db.clone(), None, None)
        .expect("Expected a successful lookup.");

    assert_eq!((&users_from_db).len(), 1);
    // assert!((&users_from_db).len() > 0);
}

#[test]
fn model_find_should_find_instances_of_model_matching_filter() {
    let fixture = Fixture::new().with_dropped_database().with_synced_models();
    let db = fixture.get_db();
    let mut user = User{id: None, email: "test@test.com".to_string()};
    user.save(db.clone(), None).expect("Expected a successful save operation.");
    let doc = doc!{"_id" => (user.id.clone().unwrap())};

    let users_from_db = User::find(db.clone(), Some(doc), None)
        .expect("Expected a successful lookup.");

    assert_eq!((&users_from_db).len(), 1);
    assert_eq!(&users_from_db[0].id, &user.id);
    assert_eq!(&users_from_db[0].email, &user.email);
}

//////////////////////////////////////////////////////////////////////////////
// Model::find_one ///////////////////////////////////////////////////////////

#[test]
fn model_find_one_should_fetch_the_model_instance_matching_given_filter() {
    let fixture = Fixture::new().with_dropped_database().with_synced_models();
    let db = fixture.get_db();
    let mut user = User{id: None, email: "test@test.com".to_string()};

    user.save(db.clone(), None).expect("Expected a successful save operation.");

    let doc = doc!{"_id" => (user.id.clone().unwrap())};
    let user_from_db = User::find_one(db.clone(), Some(doc), None)
        .expect("Expected a successful lookup.")
        .expect("Expected a populated value from backend.");

    assert_eq!(&user_from_db.id, &user.id);
    assert_eq!(&user_from_db.email, &user.email);
}

//////////////////////////////////////////////////////////////////////////////
// Model.update //////////////////////////////////////////////////////////////

#[test]
fn model_update_should_perform_expected_updates_against_self() {
    let fixture = Fixture::new().with_dropped_database().with_synced_models();
    let db = fixture.get_db();
    let mut user = User{id: None, email: String::from("test@test.com")};
    user.save(db.clone(), None).expect("Expected a successful save operation.");
    let update_doc = doc!{"$set" => doc!{"email" => "new@test.com"}};
    let mut opts = FindOneAndUpdateOptions::default();
    opts.return_document = Some(ReturnDocument::After);

    let user = user.update(db.clone(), update_doc, Some(opts))
        .expect("Expected a successful update operation.");

    assert_eq!(user.email, String::from("new@test.com"));
}

#[test]
fn model_update_should_return_error_with_invalid_update_document() {
    let fixture = Fixture::new().with_dropped_database().with_synced_models();
    let db = fixture.get_db();
    let mut user = User{id: None, email: String::from("test@test.com")};
    user.save(db.clone(), None).expect("Expected a successful save operation.");
    let update_doc = doc!{"invalid_update_key" => "should_fail"};

    let err = user.update(db.clone(), update_doc, None)
        .expect_err("Expected a successful update operation.");

    assert_eq!(err.description(), "Update only works with $ operators."); // NOTE: comes from `mongodb` lib.
}

//////////////////////////////////////////////////////////////////////////////
// Model::sync ///////////////////////////////////////////////////////////////

#[test]
fn model_sync_should_create_expected_indices_on_collection() {
    // TODO: update this fixture once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251 is fixed.
    let fixture = Fixture::new().with_synced_models().with_empty_collections();
    let db = fixture.get_db();
    let coll = db.collection(User::COLLECTION_NAME);
    let initial_indices: Vec<Document> = coll.list_indexes()
        .expect("Expected to successfully open indices cursor pre-test.")
        .filter_map(|doc_res| doc_res.ok())
        .collect();
    let initial_indices_len = initial_indices.len();

    let _ = User::sync(db.clone()).expect("Expected a successful sync operation.");
    let mut output_indices: Vec<Document> = coll.list_indexes()
        .expect("Expected to successfully open indices cursor post-test.")
        .filter_map(|doc_res| doc_res.ok())
        .collect();
    output_indices.sort_by_key(|doc| doc.get_str("name").unwrap().to_string()); // The name key will always exist and always be a string.
    let output_indices_len = output_indices.len();
    let idx1 = output_indices[0].clone();
    let idx2 = output_indices[1].clone();

    assert!(output_indices_len > initial_indices_len);
    assert_eq!(output_indices_len, 2);
    assert_eq!(&idx1, &doc!{"v": idx1.get_i32("v").unwrap(), "key": doc!{"_id": 1}, "name": "_id_", "ns": "witherTestDB.users"});
    assert_eq!(&idx2, &doc!{"v": idx2.get_i32("v").unwrap(), "unique": true, "key": doc!{"email": 1}, "name": "unique-email", "ns": "witherTestDB.users", "background": true});
}

#[test]
fn model_sync_should_create_expected_indices_on_collection_for_derived_model() {
    // TODO: update this fixture once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251 is fixed.
    let fixture = Fixture::new().with_synced_models().with_empty_collections();
    let db = fixture.get_db();
    let coll = db.collection(DerivedModel::COLLECTION_NAME);
    let initial_indices: Vec<Document> = coll.list_indexes()
        .expect("Expected to successfully open indices cursor pre-test.")
        .filter_map(|doc_res| doc_res.ok())
        .collect();
    let initial_indices_len = initial_indices.len();

    let _ = DerivedModel::sync(db.clone()).expect("Expected a successful sync operation.");
    let mut output_indices: Vec<Document> = coll.list_indexes()
        .expect("Expected to successfully open indices cursor post-test.")
        .filter_map(|doc_res| doc_res.ok())
        .collect();
    output_indices.sort_by_key(|doc| doc.get_str("name").unwrap().to_string()); // The name key will always exist and always be a string.
    let output_indices_len = output_indices.len();
    let idx1 = output_indices[0].clone();
    let idx2 = output_indices[1].clone();
    let idx3 = output_indices[2].clone();
    let idx4 = output_indices[3].clone();
    let idx5 = output_indices[4].clone();

    assert!(output_indices_len > initial_indices_len);
    assert_eq!(output_indices_len, 5);
    assert_eq!(&idx1, &doc!{"v": idx1.get_i32("v").unwrap(), "key": doc!{"_id": 1}, "name": "_id_", "ns": "witherTestDB.derivations"});
    assert_eq!(&idx2, &doc!{
        "v": idx2.get_i32("v").unwrap(),
        "unique": true,
        "key": doc!{"field0": 1},
        "name": "idx2",
        "ns": "witherTestDB.derivations",
        "background": true,
        "expireAfterSeconds": 15i32,
        "sparse": true,
    });
    assert_eq!(&idx3, &doc!{
        "v": idx3.get_i32("v").unwrap(),
        "key": doc!{"field1": -1i32, "text_field_a": -1i32, "field0": 1i32},
        "name": "idx3",
        "ns": "witherTestDB.derivations",
        "background": false,
        "sparse": false,
    });
    assert_eq!(&idx4, &doc!{
        "v": idx4.get_i32("v").unwrap(),
        "key": doc!{"_fts": "text", "_ftsx": 1i32},
        "name": "idx4",
        "ns": "witherTestDB.derivations",
        "default_language": "en",
        "language_override": "override_field",
        "weights": doc!{"text_field_a": 10i32, "text_field_b": 5i32},
        "textIndexVersion": 3i32,
    });
    assert_eq!(&idx5, &doc!{
        "v": idx5.get_i32("v").unwrap(),
        "key": doc!{"hashed_field": "hashed"},
        "name": "idx5",
        "ns": "witherTestDB.derivations",
    });
}

#[test]
fn model_sync_should_create_expected_indices_on_collection_for_derived_2d_model() {
    // TODO: update this fixture once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251 is fixed.
    let fixture = Fixture::new().with_synced_models().with_empty_collections();
    let db = fixture.get_db();
    let coll = db.collection(Derived2dModel::COLLECTION_NAME);
    let initial_indices: Vec<Document> = coll.list_indexes()
        .expect("Expected to successfully open indices cursor pre-test.")
        .filter_map(|doc_res| doc_res.ok())
        .collect();
    let initial_indices_len = initial_indices.len();

    let _ = Derived2dModel::sync(db.clone()).expect("Expected a successful sync operation.");
    let mut output_indices: Vec<Document> = coll.list_indexes()
        .expect("Expected to successfully open indices cursor post-test.")
        .filter_map(|doc_res| doc_res.ok())
        .collect();
    output_indices.sort_by_key(|doc| doc.get_str("name").unwrap().to_string()); // The name key will always exist and always be a string.
    let output_indices_len = output_indices.len();
    let idx1 = output_indices[0].clone();
    let idx2 = output_indices[1].clone();

    assert!(output_indices_len > initial_indices_len);
    assert_eq!(output_indices_len, 2);
    assert_eq!(&idx1, &doc!{"v": idx1.get_i32("v").unwrap(), "key": doc!{"_id": 1}, "name": "_id_", "ns": "witherTestDB.derived_2d_models"});
    // NOTE WELL: doc comparison was failing for some reason. Not sure why. Doing manual asserts now.
    assert_eq!(idx2.get_i32("v").unwrap(), 2i32);
    assert_eq!(idx2.get("key").unwrap().as_document().unwrap(), &doc!{"field_2d_a": "2d", "field_2d_filter": 1i32});
    assert_eq!(idx2.get("name").unwrap().as_str().unwrap(), "field_2d_a_2d_field_2d_filter_1");
    assert_eq!(idx2.get("ns").unwrap().as_str().unwrap(), "witherTestDB.derived_2d_models");
    assert_eq!(idx2.get("min").unwrap().as_f64().unwrap(), -180.0f64);
    assert_eq!(idx2.get("max").unwrap().as_f64().unwrap(), 180.0f64);
    assert_eq!(idx2.get("bits").unwrap().as_i32().unwrap(), 1i32);
}

#[test]
fn model_sync_should_create_expected_indices_on_collection_for_derived_2dsphere_model() {
    // TODO: update this fixture once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251 is fixed.
    let fixture = Fixture::new().with_synced_models().with_empty_collections();
    let db = fixture.get_db();
    let coll = db.collection(Derived2dsphereModel::COLLECTION_NAME);
    let initial_indices: Vec<Document> = coll.list_indexes()
        .expect("Expected to successfully open indices cursor pre-test.")
        .filter_map(|doc_res| doc_res.ok())
        .collect();
    let initial_indices_len = initial_indices.len();

    let _ = Derived2dsphereModel::sync(db.clone()).expect("Expected a successful sync operation.");
    let mut output_indices: Vec<Document> = coll.list_indexes()
        .expect("Expected to successfully open indices cursor post-test.")
        .filter_map(|doc_res| doc_res.ok())
        .collect();
    output_indices.sort_by_key(|doc| doc.get_str("name").unwrap().to_string()); // The name key will always exist and always be a string.
    let output_indices_len = output_indices.len();
    let idx1 = output_indices[0].clone();
    let idx2 = output_indices[1].clone();

    assert!(output_indices_len > initial_indices_len);
    assert_eq!(output_indices_len, 2);
    assert_eq!(&idx1, &doc!{"v": idx1.get_i32("v").unwrap(), "key": doc!{"_id": 1}, "name": "_id_", "ns": "witherTestDB.derived_2dsphere_models"});
    assert_eq!(idx2, doc!{
        "v": 2i32,
        "key": doc!{"field_2dsphere": "2dsphere", "field_2dsphere_filter": 1i32},
        "name": "field_2dsphere_2dsphere_field_2dsphere_filter_1",
        "ns": "witherTestDB.derived_2dsphere_models",
        "2dsphereIndexVersion": 3i32,
    });
}

// TODO: enable this test once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/289 lands.
// #[test]
// fn model_sync_should_create_expected_indices_on_collection_for_derived_haystack_model() {
//     // TODO: update this fixture once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251 is fixed.
//     let fixture = Fixture::new().with_synced_models().with_empty_collections();
//     let db = fixture.get_db();
//     let coll = db.collection(DerivedGeoHaystackModel::COLLECTION_NAME);
//     let initial_indices: Vec<Document> = coll.list_indexes()
//         .expect("Expected to successfully open indices cursor pre-test.")
//         .filter_map(|doc_res| doc_res.ok())
//         .collect();
//     let initial_indices_len = initial_indices.len();

//     let _ = DerivedGeoHaystackModel::sync(db.clone()).expect("Expected a successful sync operation.");
//     let mut output_indices: Vec<Document> = coll.list_indexes()
//         .expect("Expected to successfully open indices cursor post-test.")
//         .filter_map(|doc_res| doc_res.ok())
//         .collect();
//     output_indices.sort_by_key(|doc| doc.get_str("name").unwrap().to_string()); // The name key will always exist and always be a string.
//     let output_indices_len = output_indices.len();
//     let idx1 = output_indices[0].clone();
//     let idx2 = output_indices[1].clone();

//     assert!(output_indices_len > initial_indices_len);
//     assert_eq!(output_indices_len, 2);
//     assert_eq!(&idx1, &doc!{"v": idx1.get_i32("v").unwrap(), "key": doc!{"_id": 1}, "name": "_id_", "ns": "witherTestDB.derived_geo_haystack_models"});
//     assert_eq!(idx2, doc!{
//         "v": 2i32,
//         "key": doc!{"field_geo_haystack": "geoHaystack", "field_geo_haystack_filter": 1i32},
//         "name": "field_geo_haystack_geohaystack_field_geo_haystack_filter_1",
//         "ns": "witherTestDB.derived_geo_haystack_models",
//         "bucketSize": 5i32,
//     });
// }

#[test]
fn model_sync_should_execute_expected_migrations_against_collection() {
    let fixture = Fixture::new().with_dropped_database();
    let db = fixture.get_db();
    let coll = db.collection(User::COLLECTION_NAME);
    let mut new_user = User{id: None, email: String::from("test@test.com")};
    new_user.save(db.clone(), None).expect("Expected to successfully save new user instance.");

    let _ = User::migrate(db.clone()).expect("Expected a successful migration operation.");
    let migrated_doc = coll.find_one(Some(doc!{"_id": new_user.id.clone().unwrap()}), None)
        .expect("Expect a successful find operation.")
        .expect("Expect a populated document.");

    assert_eq!(migrated_doc, doc!{
        "_id": new_user.id.clone().unwrap(),
        "email": new_user.email,
        "testfield": "test",
    });
}

#[test]
fn model_sync_should_error_if_migration_with_no_set_and_no_unset_given() {
    let fixture = Fixture::new().with_dropped_database().with_synced_models();
    let db = fixture.get_db();
    let mut new_user = UserModelBadMigrations{id: None, email: String::from("test@test.com")};
    new_user.save(db.clone(), None).expect("Expected to successfully save new user instance.");

    let err = UserModelBadMigrations::migrate(db.clone()).expect_err("Expected a failure from migration operation.");

    assert_eq!(err.description(), "One of '$set' or '$unset' must be specified.");
}
