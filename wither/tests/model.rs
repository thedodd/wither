#![cfg(not(feature = "sync"))]

mod fixtures;

use std::collections::HashMap;

use fixtures::{models::*, Fixture, User};
use futures::stream::StreamExt;
use wither::bson::doc;
use wither::mongodb::options::{FindOneAndReplaceOptions, FindOneAndUpdateOptions, ReturnDocument};
use wither::{prelude::*, IndexModel};

//////////////////////////////////////////////////////////////////////////////
// Model::find ///////////////////////////////////////////////////////////////
#[tokio::test]
async fn model_find_should_find_all_instances_of_model_with_no_filter_or_options() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: "test@test.com".to_string(),
    };
    user.save(&db, None).await.expect("Expected a successful save operation.");

    let mut users_from_db: Vec<_> = User::find(&db, None, None).await.expect("Expected a successful lookup.").collect().await;

    assert_eq!(users_from_db.len(), 1);
    let userdb = users_from_db.pop().unwrap();
    assert!(userdb.is_ok());
    let userdb = userdb.unwrap();
    user.id = userdb.id.clone();
    assert_eq!(userdb, user);
}

#[tokio::test]
async fn model_find_should_find_instances_of_model_matching_filter() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: "test@test.com".to_string(),
    };
    user.save(&db, None).await.expect("Expected a successful save operation.");
    let doc = doc! {"_id": (user.id.clone().unwrap())};

    let mut users_from_db: Vec<_> = User::find(&db, Some(doc), None)
        .await
        .expect("Expected a successful lookup.")
        .collect()
        .await;

    assert_eq!(users_from_db.len(), 1);
    let userdb = users_from_db.pop().unwrap();
    assert!(userdb.is_ok());
    let userdb = userdb.unwrap();
    user.id = userdb.id.clone();
    assert_eq!(userdb, user);
}

//////////////////////////////////////////////////////////////////////////////
// Model::find_one ///////////////////////////////////////////////////////////

#[tokio::test]
async fn model_find_one_should_fetch_the_model_instance_matching_given_filter() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: "test@test.com".to_string(),
    };

    user.save(&db, None).await.expect("Expected a successful save operation.");

    let doc = doc! {"_id": (user.id.clone().unwrap())};
    let user_from_db = User::find_one(&db, Some(doc), None)
        .await
        .expect("Expected a successful lookup.")
        .expect("Expected a populated value from backend.");

    assert_eq!(&user_from_db.id, &user.id);
    assert_eq!(&user_from_db.email, &user.email);
}

//////////////////////////////////////////////////////////////////////////////
// Model::find_one_and_delete ////////////////////////////////////////////////

#[tokio::test]
async fn model_find_one_and_delete_should_delete_the_target_doc() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: "test@test.com".to_string(),
    };
    let mut user2 = User {
        id: None,
        email: "test2@test.com".to_string(),
    };

    user.save(&db, None).await.expect("Expected a successful save operation.");
    user2.save(&db, None).await.expect("Expected a successful save operation.");
    let output = User::find_one_and_delete(&db, doc! {"email": "test@test.com"}, None)
        .await
        .expect("Expected a operation.")
        .unwrap();

    assert_eq!(&output.email, &user.email);
}

//////////////////////////////////////////////////////////////////////////////
// Model::find_one_and_replace ///////////////////////////////////////////////

#[tokio::test]
async fn model_find_one_and_replace_should_replace_the_target_doc_and_return_new_doc() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: "test@test.com".to_string(),
    };
    let mut user2 = User {
        id: None,
        email: "test2@test.com".to_string(),
    };
    let mut opts = FindOneAndReplaceOptions::builder().build();
    opts.return_document = Some(ReturnDocument::After);

    user.save(&db, None).await.expect("Expected a successful save operation.");
    user2.save(&db, None).await.expect("Expected a successful save operation.");
    let output = User::find_one_and_replace(&db, doc! {"email": "test@test.com"}, doc! {"email": "test3@test.com"}, Some(opts))
        .await
        .expect("Expected a operation.")
        .unwrap();

    assert_eq!(&output.email, "test3@test.com");
}

#[tokio::test]
async fn model_find_one_and_replace_should_replace_the_target_doc_and_return_old_doc() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: "test@test.com".to_string(),
    };
    let mut user2 = User {
        id: None,
        email: "test2@test.com".to_string(),
    };
    let mut opts = FindOneAndReplaceOptions::builder().build();
    opts.return_document = Some(ReturnDocument::Before);

    user.save(&db, None).await.expect("Expected a successful save operation.");
    user2.save(&db, None).await.expect("Expected a successful save operation.");
    let output = User::find_one_and_replace(&db, doc! {"email": "test@test.com"}, doc! {"email": "test3@test.com"}, Some(opts))
        .await
        .expect("Expected a operation.")
        .unwrap();

    assert_eq!(&output.email, "test@test.com");
}

//////////////////////////////////////////////////////////////////////////////
// Model::find_one_and_update ////////////////////////////////////////////////

#[tokio::test]
async fn model_find_one_and_update_should_update_target_document_and_return_new() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: "test@test.com".to_string(),
    };
    let mut user2 = User {
        id: None,
        email: "test2@test.com".to_string(),
    };
    let mut opts = FindOneAndUpdateOptions::builder().build();
    opts.return_document = Some(ReturnDocument::After);

    user.save(&db, None).await.expect("Expected a successful save operation.");
    user2.save(&db, None).await.expect("Expected a successful save operation.");
    let output = User::find_one_and_update(
        &db,
        doc! {"email": "test@test.com"},
        doc! {"$set": doc!{"email": "test3@test.com"}},
        Some(opts),
    )
    .await
    .expect("Expected a operation.")
    .unwrap();

    assert_eq!(&output.email, "test3@test.com");
}

#[tokio::test]
async fn model_find_one_and_update_should_update_target_document_and_return_old() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: "test@test.com".to_string(),
    };
    let mut user2 = User {
        id: None,
        email: "test2@test.com".to_string(),
    };
    let mut opts = FindOneAndUpdateOptions::builder().build();
    opts.return_document = Some(ReturnDocument::Before);

    user.save(&db, None).await.expect("Expected a successful save operation.");
    user2.save(&db, None).await.expect("Expected a successful save operation.");
    let output = User::find_one_and_update(
        &db,
        doc! {"email": "test@test.com"},
        doc! {"$set": doc!{"email": "test3@test.com"}},
        Some(opts),
    )
    .await
    .expect("Expected a operation.")
    .unwrap();

    assert_eq!(&output.email, "test@test.com");
}

//////////////////////////////////////////////////////////////////////////////
// Model.save ////////////////////////////////////////////////////////////////

#[tokio::test]
async fn model_save_should_save_model_instance_and_add_id() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: "test@test.com".to_string(),
    };

    let precount = User::collection(&db).count_documents(None, None).await.unwrap();
    user.save(&db, None).await.expect("Expected a successful save operation.");
    let postcount = User::collection(&db).count_documents(None, None).await.unwrap();

    assert!(user.id != None);
    assert_eq!(precount, 0);
    assert_eq!(postcount, 1);
    assert!(precount != postcount);
}

//////////////////////////////////////////////////////////////////////////////
// Model.update //////////////////////////////////////////////////////////////

#[tokio::test]
async fn model_update_should_perform_expected_updates_against_self() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: String::from("test@test.com"),
    };
    user.save(&db, None).await.expect("Expected a successful save operation.");
    let update_doc = doc! {"$set": doc!{"email": "new@test.com"}};
    let mut opts = FindOneAndUpdateOptions::default();
    opts.return_document = Some(ReturnDocument::After);

    let user = user
        .update(&db, None, update_doc, Some(opts))
        .await
        .expect("Expected a successful update operation.");

    assert_eq!(user.email, String::from("new@test.com"));
}

#[tokio::test]
async fn model_update_should_return_error_with_invalid_update_document() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: String::from("test@test.com"),
    };
    user.save(&db, None).await.expect("Expected a successful save operation.");
    let update_doc = doc! {"invalid_update_key": "should_fail"};

    let err = user
        .update(&db, None, update_doc, None)
        .await
        .expect_err("Expected an errored update operation.");

    assert_eq!(
        err.to_string(),
        "An invalid argument was provided to a database operation: update document must have first key starting with '$"
    );
}

#[tokio::test]
async fn model_update_should_noop_where_filter_selects_on_nonextant_field() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: String::from("test@test.com"),
    };
    user.save(&db, None).await.expect("Expected a successful save operation.");
    let filter_doc = Some(doc! {"nonextant_field": doc!{"$exists": true}});
    let update_doc = doc! {"$set": doc!{"email": "test2@test.com"}};

    let res = user.update(&db, filter_doc, update_doc, None).await;

    assert!(res.is_err());
}

#[tokio::test]
async fn model_update_should_perform_expected_update_with_added_filters() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: String::from("test@test.com"),
    };
    user.save(&db, None).await.expect("Expected a successful save operation.");
    let filter_doc = Some(doc! {"nonextant_field": doc!{"$exists": false}});
    let update_doc = doc! {"$set": doc!{"email": "test2@test.com"}};
    let mut opts = FindOneAndUpdateOptions::default();
    opts.return_document = Some(ReturnDocument::After);

    let user = user
        .update(&db, filter_doc, update_doc, Some(opts))
        .await
        .expect("Expected a successful update operation.");

    assert_eq!(user.email, String::from("test2@test.com"));
}

//////////////////////////////////////////////////////////////////////////////
// Model.delete //////////////////////////////////////////////////////////////

#[tokio::test]
async fn model_delete_should_delete_model_instance() {
    let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
    let db = fixture.get_db();
    let mut user = User {
        id: None,
        email: "test@test.com".to_string(),
    };

    let presave = User::collection(&db).count_documents(None, None).await.unwrap();
    user.save(&db, None).await.expect("Expected a successful save operation.");
    let postsave = User::collection(&db).count_documents(None, None).await.unwrap();
    user.delete(&db).await.unwrap();
    let postdelete = User::collection(&db).count_documents(None, None).await.unwrap();

    assert_eq!(presave, 0);
    assert_eq!(postsave, 1);
    assert_eq!(postdelete, 0);
    assert!(presave != postsave);
    assert!(postsave != postdelete);
}

//////////////////////////////////////////////////////////////////////////////
// Model::sync ///////////////////////////////////////////////////////////////

#[tokio::test]
async fn model_sync_should_create_expected_indices_on_collection() {
    let fixture = Fixture::new().await.with_dropped_database().await;
    let db = fixture.get_db();

    // There should be no indexes now
    let before_indexes: HashMap<String, IndexModel> = IndexTestV1::get_current_indexes(&db).await;
    assert!(before_indexes.is_empty());

    // It should sync the model
    IndexTestV1::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV1::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_1").expect("Should have index: `i_1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, 1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_1");
}

#[tokio::test]
async fn model_sync_should_not_modify_indexes() {
    let fixture = Fixture::new().await.with_dropped_database().await;
    let db = fixture.get_db();

    // There should be no indexes now
    let before_indexes: HashMap<String, IndexModel> = IndexTestV1::get_current_indexes(&db).await;
    assert!(before_indexes.is_empty());

    IndexTestV1::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV1::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_1").expect("Should have index: `i_1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, 1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_1");

    IndexTestV1::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV1::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_1").expect("Should have index: `i_1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, 1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_1");
}

#[tokio::test]
async fn model_sync_should_modify_indexes_v1_to_v2() {
    let fixture = Fixture::new().await.with_dropped_database().await;
    let db = fixture.get_db();

    // There should be no indexes now
    let before_indexes: HashMap<String, IndexModel> = IndexTestV1::get_current_indexes(&db).await;
    assert!(before_indexes.is_empty());

    IndexTestV1::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV1::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_1").expect("Should have index: `i_1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, 1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_1");

    IndexTestV2::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV1::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_-1").expect("Should have index: `i_-1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, -1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_-1");
}

#[tokio::test]
async fn model_sync_should_modify_indexes_v2_to_v3() {
    let fixture = Fixture::new().await.with_dropped_database().await;
    let db = fixture.get_db();

    // There should be no indexes now
    let before_indexes: HashMap<String, IndexModel> = IndexTestV2::get_current_indexes(&db).await;
    assert!(before_indexes.is_empty());

    IndexTestV2::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV2::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_-1").expect("Should have index: `i_-1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, -1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_-1");

    IndexTestV3::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV3::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_-1").expect("Should have index: `i_-1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, -1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_-1");

    let option_unique_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("unique")
        .expect("Should have unique option")
        .as_bool()
        .expect("Should be a valid boolean");

    assert!(option_unique_value);
}

#[tokio::test]
async fn model_sync_should_modify_indexes_v3_to_v4() {
    let fixture = Fixture::new().await.with_dropped_database().await;
    let db = fixture.get_db();

    // There should be no indexes now
    let before_indexes: HashMap<String, IndexModel> = IndexTestV3::get_current_indexes(&db).await;
    assert!(before_indexes.is_empty());

    IndexTestV3::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV3::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_-1").expect("Should have index: `i_-1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, -1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_-1");

    let option_unique_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("unique")
        .expect("Should have unique option")
        .as_bool()
        .expect("Should be a valid boolean");

    assert!(option_unique_value);

    IndexTestV4::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV4::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_-1").expect("Should have index: `i_-1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, -1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_-1");

    let option_unique_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("unique")
        .expect("Should have unique option")
        .as_bool()
        .expect("Should be a valid boolean");

    assert!(option_unique_value);

    let option_background_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("background")
        .expect("Should have background option")
        .as_bool()
        .expect("Should be a valid boolean");

    assert!(option_background_value);
}

#[tokio::test]
async fn model_sync_should_modify_indexes_v4_to_v5() {
    let fixture = Fixture::new().await.with_dropped_database().await;
    let db = fixture.get_db();

    IndexTestV4::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV4::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_-1").expect("Should have index: `i_-1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, -1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_-1");

    let option_unique_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("unique")
        .expect("Should have unique option")
        .as_bool()
        .expect("Should be a valid boolean");

    assert!(option_unique_value);

    let option_background_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("background")
        .expect("Should have background option")
        .as_bool()
        .expect("Should be a valid boolean");

    assert!(option_background_value);
    IndexTestV4::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV4::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_-1").expect("Should have index: `i_-1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, -1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_-1");

    let option_unique_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("unique")
        .expect("Should have unique option")
        .as_bool()
        .expect("Should be a valid boolean");

    assert!(option_unique_value);

    let option_background_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("background")
        .expect("Should have background option")
        .as_bool()
        .expect("Should be a valid boolean");

    assert!(option_background_value);
}

#[tokio::test]
async fn model_sync_should_modify_indexes_v5_to_v6() {
    let fixture = Fixture::new().await.with_dropped_database().await;
    let db = fixture.get_db();

    IndexTestV5::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV5::get_current_indexes(&db).await;
    let index_model = after_indexes.get("i_-1").expect("Should have index: `i_-1`");
    let index_key_value = index_model
        .keys
        .get("i")
        .expect("Should have key `i`")
        .as_i32()
        .expect("`i` should be of type Int32");

    assert_eq!(index_key_value, -1);

    let option_name_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("name")
        .expect("Should have name option")
        .as_str()
        .expect("Should be a valid string");

    assert_eq!(option_name_value, "i_-1");

    let option_unique_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("unique")
        .expect("Should have unique option")
        .as_bool()
        .expect("Should be a valid boolean");

    assert!(option_unique_value);

    let option_background_value = index_model
        .options
        .as_ref()
        .expect("options should not be empty")
        .get("background")
        .expect("Should have background option")
        .as_bool()
        .expect("Should be a valid boolean");

    assert!(option_background_value);

    IndexTestV6::sync(&db).await.expect("Expected a successful sync operation.");

    // After the sync there should be the expected indexes
    let after_indexes: HashMap<String, IndexModel> = IndexTestV6::get_current_indexes(&db).await;

    assert!(after_indexes.is_empty());
}

// #[tokio::test]
//
// fn model_sync_should_create_expected_indices_on_collection() {
//     // TODO: update this fixture once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251 is fixed.
//     let fixture = Fixture::new().await.with_synced_models().await.with_empty_collections();
//     let db = fixture.get_db();
//     let coll = db.collection(User::COLLECTION_NAME);
//     let initial_indices: Vec<Document> = coll.list_indexes()
//         .await.expect("Expected to successfully open indices cursor pre-test.")
//         .filter_map(|doc_res| doc_res.ok())
//         .collect();
//     let initial_indices_len = initial_indices.len();

//     let _ = User::sync(&db).await.expect("Expected a successful sync operation.");
//     let mut output_indices: Vec<Document> = coll.list_indexes()
//         .await.expect("Expected to successfully open indices cursor post-test.")
//         .filter_map(|doc_res| doc_res.ok())
//         .collect();
//     output_indices.sort_by_key(|doc| doc.get_str("name").await.unwrap().to_string()); // The name
// key will always exist and always be a string.     let output_indices_len = output_indices.len();
//     let idx1 = output_indices[0].clone();
//     let idx2 = output_indices[1].clone();

//     assert!(output_indices_len > initial_indices_len);
//     assert_eq!(output_indices_len, 2);
//     assert_eq!(&idx1, &doc!{"v": idx1.get_i32("v").await.unwrap(), "key": doc!{"_id": 1}, "name":
// "_id_", "ns": "witherTestDB.users"});     assert_eq!(&idx2, &doc!{"v":
// idx2.get_i32("v").await.unwrap(), "unique": true, "key": doc!{"email": 1}, "name":
// "unique-email", "ns": "witherTestDB.users", "background": true}); }

// #[tokio::test]
//
// fn model_sync_should_create_expected_indices_on_collection_for_derived_model() {
//     // TODO: update this fixture once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251 is fixed.
//     let fixture = Fixture::new().await.with_synced_models().await.with_empty_collections();
//     let db = fixture.get_db();
//     let coll = db.collection(DerivedModel::COLLECTION_NAME);
//     let initial_indices: Vec<Document> = coll.list_indexes()
//         .await.expect("Expected to successfully open indices cursor pre-test.")
//         .filter_map(|doc_res| doc_res.ok())
//         .collect();
//     let initial_indices_len = initial_indices.len();

//     let _ = DerivedModel::sync(&db).await.expect("Expected a successful sync operation.");
//     let mut output_indices: Vec<Document> = coll.list_indexes()
//         .await.expect("Expected to successfully open indices cursor post-test.")
//         .filter_map(|doc_res| doc_res.ok())
//         .collect();
//     output_indices.sort_by_key(|doc| doc.get_str("name").await.unwrap().to_string()); // The name
// key will always exist and always be a string.     let output_indices_len = output_indices.len();
//     let idx1 = output_indices[0].clone();
//     let idx2 = output_indices[1].clone();
//     let idx3 = output_indices[2].clone();
//     let idx4 = output_indices[3].clone();
//     let idx5 = output_indices[4].clone();

//     assert!(output_indices_len > initial_indices_len);
//     assert_eq!(output_indices_len, 5);
//     assert_eq!(&idx1, &doc!{"v": idx1.get_i32("v").await.unwrap(), "key": doc!{"_id": 1}, "name":
// "_id_", "ns": "witherTestDB.derivations"});     assert_eq!(&idx2, &doc!{
//         "v": idx2.get_i32("v").await.unwrap(),
//         "unique": true,
//         "key": doc!{"field0": 1},
//         "name": "idx2",
//         "ns": "witherTestDB.derivations",
//         "background": true,
//         "expireAfterSeconds": 15i32,
//         "sparse": true,
//     });
//     assert_eq!(&idx3, &doc!{
//         "v": idx3.get_i32("v").await.unwrap(),
//         "key": doc!{"field1": -1i32, "text_field_a": -1i32, "field0": 1i32},
//         "name": "idx3",
//         "ns": "witherTestDB.derivations",
//         "background": false,
//         "sparse": false,
//     });
//     assert_eq!(&idx4, &doc!{
//         "v": idx4.get_i32("v").await.unwrap(),
//         "key": doc!{"_fts": "text", "_ftsx": 1i32},
//         "name": "idx4",
//         "ns": "witherTestDB.derivations",
//         "default_language": "en",
//         "language_override": "override_field",
//         "weights": doc!{"text_field_a": 10i32, "text_field_b": 5i32},
//         "textIndexVersion": 3i32,
//     });
//     assert_eq!(&idx5, &doc!{
//         "v": idx5.get_i32("v").await.unwrap(),
//         "key": doc!{"hashed_field": "hashed"},
//         "name": "idx5",
//         "ns": "witherTestDB.derivations",
//     });
// }

// #[tokio::test]
//
// fn model_sync_should_create_expected_indices_on_collection_for_derived_2d_model() {
//     // TODO: update this fixture once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251 is fixed.
//     let fixture = Fixture::new().await.with_synced_models().await.with_empty_collections();
//     let db = fixture.get_db();
//     let coll = db.collection(Derived2dModel::COLLECTION_NAME);
//     let initial_indices: Vec<Document> = coll.list_indexes()
//         .await.expect("Expected to successfully open indices cursor pre-test.")
//         .filter_map(|doc_res| doc_res.ok())
//         .collect();
//     let initial_indices_len = initial_indices.len();

//     let _ = Derived2dModel::sync(&db).await.expect("Expected a successful sync operation.");
//     let mut output_indices: Vec<Document> = coll.list_indexes()
//         .await.expect("Expected to successfully open indices cursor post-test.")
//         .filter_map(|doc_res| doc_res.ok())
//         .collect();
//     output_indices.sort_by_key(|doc| doc.get_str("name").await.unwrap().to_string()); // The name
// key will always exist and always be a string.     let output_indices_len = output_indices.len();
//     let idx1 = output_indices[0].clone();
//     let idx2 = output_indices[1].clone();

//     assert!(output_indices_len > initial_indices_len);
//     assert_eq!(output_indices_len, 2);
//     assert_eq!(&idx1, &doc!{"v": idx1.get_i32("v").await.unwrap(), "key": doc!{"_id": 1}, "name":
// "_id_", "ns": "witherTestDB.derived_2d_models"});     // NOTE WELL: doc comparison was failing
// for some reason. Not sure why. Doing manual asserts now.     assert_eq!(idx2.get("key").await.
// unwrap().as_document().await.unwrap(), &doc!{"field_2d_a": "2d", "field_2d_filter": 1i32});
//     assert_eq!(idx2.get("name").await.unwrap().as_str().await.unwrap(),
// "field_2d_a_2d_field_2d_filter_1");     assert_eq!(idx2.get("ns").await.unwrap().as_str().await.
// unwrap(), "witherTestDB.derived_2d_models");     assert_eq!(idx2.get("min").await.unwrap().
// as_f64().await.unwrap(), -180.0f64);     assert_eq!(idx2.get("max").await.unwrap().as_f64().
// await.unwrap(), 180.0f64);     assert_eq!(idx2.get("bits").await.unwrap().as_i32().await.
// unwrap(), 1i32); }

// #[tokio::test]
//
// fn model_sync_should_create_expected_indices_on_collection_for_derived_2dsphere_model() {
//     // TODO: update this fixture once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251 is fixed.
//     let fixture = Fixture::new().await.with_synced_models().await.with_empty_collections();
//     let db = fixture.get_db();
//     let coll = db.collection(Derived2dsphereModel::COLLECTION_NAME);
//     let initial_indices: Vec<Document> = coll.list_indexes()
//         .await.expect("Expected to successfully open indices cursor pre-test.")
//         .filter_map(|doc_res| doc_res.ok())
//         .collect();
//     let initial_indices_len = initial_indices.len();

//     let _ = Derived2dsphereModel::sync(&db).await.expect("Expected a successful sync
// operation.");     let mut output_indices: Vec<Document> = coll.list_indexes()
//         .await.expect("Expected to successfully open indices cursor post-test.")
//         .filter_map(|doc_res| doc_res.ok())
//         .collect();
//     output_indices.sort_by_key(|doc| doc.get_str("name").await.unwrap().to_string()); // The name
// key will always exist and always be a string.     let output_indices_len = output_indices.len();
//     let idx1 = output_indices[0].clone();
//     let idx2 = output_indices[1].clone();

//     assert!(output_indices_len > initial_indices_len);
//     assert_eq!(output_indices_len, 2);
//     assert_eq!(&idx1, &doc!{"v": idx1.get_i32("v").await.unwrap(), "key": doc!{"_id": 1}, "name":
// "_id_", "ns": "witherTestDB.derived_2dsphere_models"});     assert_eq!(idx2.get("key").await.
// unwrap().as_document().await.unwrap(), &doc!{"field_2dsphere": "2dsphere",
// "field_2dsphere_filter": 1i32});     assert_eq!(idx2.get("name").await.unwrap().as_str().await.
// unwrap(), "field_2dsphere_2dsphere_field_2dsphere_filter_1");     assert_eq!(idx2.get("ns").
// await.unwrap().as_str().await.unwrap(), "witherTestDB.derived_2dsphere_models");     assert_eq!
// (idx2.get("2dsphereIndexVersion").await.unwrap().as_i32().await.unwrap(), 3i32); }

// // TODO: enable this test once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/289 lands.
// // #[tokio::test]
//
// // fn model_sync_should_create_expected_indices_on_collection_for_derived_haystack_model() {
// //     // TODO: update this fixture once https://github.com/mongodb-labs/mongo-rust-driver-prototype/issues/251 is fixed.
// //     let fixture = Fixture::new().await.with_synced_models().await.with_empty_collections();
// //     let db = fixture.get_db();
// //     let coll = db.collection(DerivedGeoHaystackModel::COLLECTION_NAME);
// //     let initial_indices: Vec<Document> = coll.list_indexes()
// //         .await.expect("Expected to successfully open indices cursor pre-test.")
// //         .filter_map(|doc_res| doc_res.ok())
// //         .collect();
// //     let initial_indices_len = initial_indices.len();

// //     let _ = DerivedGeoHaystackModel::sync(&db).await.expect("Expected a successful sync
// operation."); //     let mut output_indices: Vec<Document> = coll.list_indexes()
// //         .await.expect("Expected to successfully open indices cursor post-test.")
// //         .filter_map(|doc_res| doc_res.ok())
// //         .collect();
// //     output_indices.sort_by_key(|doc| doc.get_str("name").await.unwrap().to_string()); // The
// name key will always exist and always be a string. //     let output_indices_len =
// output_indices.len(); //     let idx1 = output_indices[0].clone();
// //     let idx2 = output_indices[1].clone();

// //     assert!(output_indices_len > initial_indices_len);
// //     assert_eq!(output_indices_len, 2);
// //     assert_eq!(&idx1, &doc!{"v": idx1.get_i32("v").await.unwrap(), "key": doc!{"_id": 1},
// "name": "_id_", "ns": "witherTestDB.derived_geo_haystack_models"}); //     assert_eq!(idx2, doc!{
// //         "v": 2i32,
// //         "key": doc!{"field_geo_haystack": "geoHaystack", "field_geo_haystack_filter": 1i32},
// //         "name": "field_geo_haystack_geohaystack_field_geo_haystack_filter_1",
// //         "ns": "witherTestDB.derived_geo_haystack_models",
// //         "bucketSize": 5i32,
// //     });
// // }

// #[tokio::test]
//
// fn model_sync_should_execute_expected_migrations_against_collection() {
//     let fixture = Fixture::new().await.with_dropped_database().await;
//     let db = fixture.get_db();
//     let coll = db.collection(User::COLLECTION_NAME);
//     let mut new_user = User{id: None, email: String::from("test@test.com")};
//     new_user.save(&db, None).await.expect("Expected to successfully save new user instance.");

//     let _ = User::migrate(&db).await.expect("Expected a successful migration operation.");
//     let migrated_doc = coll.find_one(Some(doc!{"_id": new_user.id.clone().await.unwrap()}), None)
//         .await.expect("Expect a successful find operation.")
//         .await.expect("Expect a populated document.");

//     assert_eq!(migrated_doc, doc!{
//         "_id": new_user.id.clone().await.unwrap(),
//         "email": new_user.email,
//         "testfield": "test",
//     });
// }

// #[tokio::test]
//
// fn model_sync_should_error_if_migration_with_no_set_and_no_unset_given() {
//     let fixture = Fixture::new().await.with_dropped_database().await.with_synced_models().await;
//     let db = fixture.get_db();
//     let mut new_user = UserModelBadMigrations{id: None, email: String::from("test@test.com")};
//     new_user.save(&db, None).await.expect("Expected to successfully save new user instance.");

//     let err = UserModelBadMigrations::migrate(&db).expect_err("Expected a failure from migration
// operation.");

//     assert_eq!(err.description(), "One of '$set' or '$unset' must be specified.");
// }
