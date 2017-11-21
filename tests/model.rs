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
use wither::Model;

use fixtures::{setup, User};

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

// #[test]
// fn model_sync_should_create_expected_indices_on_collection() {
//     let db = setup();
//     let coll = db.collection(User::COLLECTION_NAME);
//     let initial_indices = coll.list_indexes().expect("Expected to successfully list indices.");
//     // let mut user = User{id: None, email: String::from("test@test.com")};
//     // user.save(db.clone(), None).expect("Expected a successful save operation.");
//     // let update_doc = doc!{"invalid_update_key" => "should_fail"};
//     //
//     // let err = user.update(db.clone(), update_doc, None)
//     //     .expect_err("Expected a successful update operation.");
//     //
//     // assert_eq!(err.description(), "Update only works with $ operators."); // NOTE: comes from `mongodb` lib.
// }
