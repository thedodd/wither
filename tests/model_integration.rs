#[macro_use(doc, bson)]
extern crate bson;
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate wither;

use mongodb::coll::options::IndexModel;
use mongodb::ThreadedClient;
use wither::Model;

static TEST_DB: &'static str = "witherTestDB";
static BACKEND_ERR_MSG: &'static str = "Expected MongoDB instance to be available for testing.";


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// The user's unique ID.
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,

    /// The user's unique email.
    pub email: String,
}

impl<'a> wither::Model<'a> for User {
    type model = User;

    const COLLECTION_NAME: &'static str = "users";

    fn id(&self) -> Option<bson::oid::ObjectId> {
        return self.id.clone();
    }

    fn set_id(&mut self, oid: bson::oid::ObjectId) {
        self.id = Some(oid);
    }

    fn indexes() -> Vec<IndexModel> {
        return vec![
            IndexModel{
                keys: doc!{"email" => 1},
                options: wither::basic_index_options("unique-email", true, Some(true), None, None),
            },
        ];
    }
}

#[test]
fn model_save_should_save_model_instance_and_add_id() {
    let client = mongodb::Client::with_uri("mongodb://mongodb.3-4:27017/").expect(BACKEND_ERR_MSG);
    let db = client.db(TEST_DB);
    let mut user = User{id: None, email: "test@test.com".to_string()};

    user.save(db.clone(), None).expect("Expected a successful save operation.");

    assert!(user.id != None)
}

#[test]
fn model_find_one_should_fetch_the_model_instance_matching_given_filter() {
    let client = mongodb::Client::with_uri("mongodb://mongodb.3-4:27017/").expect(BACKEND_ERR_MSG);
    let db = client.db(TEST_DB);
    let mut user = User{id: None, email: "test@test.com".to_string()};

    user.save(db.clone(), None).expect("Expected a successful save operation.");

    let doc = doc!{"_id" => (user.id.clone().unwrap())};
    let user_from_db = User::find_one(db.clone(), Some(doc), None)
        .expect("Expected a successful lookup.")
        .expect("Expected a populated value from backend.");

    assert_eq!(&user_from_db.id, &user.id);
    assert_eq!(&user_from_db.email, &user.email);
}

#[test]
fn model_find_should_find_all_instances_of_model_with_no_filter_or_options() {
    let client = mongodb::Client::with_uri("mongodb://mongodb.3-4:27017/").expect(BACKEND_ERR_MSG);
    let db = client.db(TEST_DB);
    let mut user = User{id: None, email: "test@test.com".to_string()};
    user.save(db.clone(), None).expect("Expected a successful save operation.");

    let users_from_db = User::find(db.clone(), None, None)
        .expect("Expected a successful lookup.");

    assert_eq!((&users_from_db).len(), 1);
}

#[test]
fn model_find_should_find_instances_of_model_matching_filter() {
    let client = mongodb::Client::with_uri("mongodb://mongodb.3-4:27017/").expect(BACKEND_ERR_MSG);
    let db = client.db(TEST_DB);
    let mut user = User{id: None, email: "test@test.com".to_string()};
    user.save(db.clone(), None).expect("Expected a successful save operation.");
    let doc = doc!{"_id" => (user.id.clone().unwrap())};

    let users_from_db = User::find(db.clone(), Some(doc), None)
        .expect("Expected a successful lookup.");

    assert_eq!((&users_from_db).len(), 1);
    assert_eq!(&users_from_db[0].id, &user.id);
    assert_eq!(&users_from_db[0].email, &user.email);
}
