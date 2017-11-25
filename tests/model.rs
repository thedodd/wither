#[macro_use(doc, bson)]
extern crate bson;
extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;

mod fixtures;

use std::error::Error;

use mongodb::coll::options::{FindOneAndUpdateOptions, ReturnDocument};
use mongodb::db::ThreadedDatabase;
use wither::Model;

use fixtures::{setup, User, UserModelBadMigrations};

//////////////////
// Model::count //

#[test]
fn model_count_should_return_expected_count_matching_filter() {
    let db = setup();
    let mut user = User{id: None, email: "test@test.com".to_string()};
    user.save(db.clone(), None).expect("Expected a successful save operation.");
    let doc = doc!{"_id" => (user.id.clone().unwrap())};

    let count = User::count(db.clone(), Some(doc), None)
        .expect("Expected a successful count operation.");

    assert_eq!(count, 1);
}

////////////////
// Model.save //

#[test]
fn model_save_should_save_model_instance_and_add_id() {
    let db = setup();
    let mut user = User{id: None, email: "test@test.com".to_string()};

    user.save(db.clone(), None).expect("Expected a successful save operation.");

    assert!(user.id != None)
}

/////////////////
// Model::find //

#[test]
fn model_find_should_find_all_instances_of_model_with_no_filter_or_options() {
    let db = setup();
    let mut user = User{id: None, email: "test@test.com".to_string()};
    user.save(db.clone(), None).expect("Expected a successful save operation.");

    let users_from_db = User::find(db.clone(), None, None)
        .expect("Expected a successful lookup.");

    assert_eq!((&users_from_db).len(), 1);
    // assert!((&users_from_db).len() > 0);
}

#[test]
fn model_find_should_find_instances_of_model_matching_filter() {
    let db = setup();
    let mut user = User{id: None, email: "test@test.com".to_string()};
    user.save(db.clone(), None).expect("Expected a successful save operation.");
    let doc = doc!{"_id" => (user.id.clone().unwrap())};

    let users_from_db = User::find(db.clone(), Some(doc), None)
        .expect("Expected a successful lookup.");

    assert_eq!((&users_from_db).len(), 1);
    assert_eq!(&users_from_db[0].id, &user.id);
    assert_eq!(&users_from_db[0].email, &user.email);
}

/////////////////////
// Model::find_one //

#[test]
fn model_find_one_should_fetch_the_model_instance_matching_given_filter() {
    let db = setup();
    let mut user = User{id: None, email: "test@test.com".to_string()};

    user.save(db.clone(), None).expect("Expected a successful save operation.");

    let doc = doc!{"_id" => (user.id.clone().unwrap())};
    let user_from_db = User::find_one(db.clone(), Some(doc), None)
        .expect("Expected a successful lookup.")
        .expect("Expected a populated value from backend.");

    assert_eq!(&user_from_db.id, &user.id);
    assert_eq!(&user_from_db.email, &user.email);
}

//////////////////
// Model.update //

#[test]
fn model_update_should_perform_expected_updates_against_self() {
    let db = setup();
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
    let db = setup();
    let mut user = User{id: None, email: String::from("test@test.com")};
    user.save(db.clone(), None).expect("Expected a successful save operation.");
    let update_doc = doc!{"invalid_update_key" => "should_fail"};

    let err = user.update(db.clone(), update_doc, None)
        .expect_err("Expected a successful update operation.");

    assert_eq!(err.description(), "Update only works with $ operators."); // NOTE: comes from `mongodb` lib.
}

/////////////////
// Model::sync //

#[test]
fn model_sync_should_create_expected_indices_on_collection() {
    let db = setup();
    let coll = db.collection(User::COLLECTION_NAME);
    let initial_indices: Vec<bson::Document> = coll.list_indexes()
        .expect("Expected to successfully open indices cursor pre-test.")
        .filter_map(|doc_res| doc_res.ok())
        .collect();
    let initial_indices_len = initial_indices.len();

    let _ = User::sync(db.clone()).expect("Expected a successful sync operation.");
    let output_indices: Vec<bson::Document> = coll.list_indexes()
        .expect("Expected to successfully open indices cursor post-test.")
        .filter_map(|doc_res| doc_res.ok())
        .collect();
    let output_indices_len = output_indices.len();
    let new_idx = output_indices[1].clone();

    assert!(output_indices_len.clone() > initial_indices_len.clone());
    assert_eq!(output_indices_len.clone(), 2);
    assert_eq!(&new_idx, &doc!{
        "v": 2i32,
        "unique": true,
        "key": doc!{"email": 1},
        "name": "unique-email",
        "ns": "witherTestDB.users",
        "background": true,
    });
}

#[test]
fn model_sync_should_execute_expected_migrations_against_collection() {
    let db = setup();
    let coll = db.collection(User::COLLECTION_NAME);
    let mut new_user = User{id: None, email: String::from("test@test.com")};
    new_user.save(db.clone(), None).expect("Expected to successfully save new user instance.");

    let _ = User::sync(db.clone()).expect("Expected a successful sync operation.");
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
    let db = setup();
    let mut new_user = UserModelBadMigrations{id: None, email: String::from("test@test.com")};
    new_user.save(db.clone(), None).expect("Expected to successfully save new user instance.");

    let err = UserModelBadMigrations::sync(db.clone()).expect_err("Expected a failure from sync operation.");

    assert_eq!(err.description(), "One of '$set' or '$unset' must be specified.");
}
