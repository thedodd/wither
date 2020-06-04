use serde::{Serialize, Deserialize};
use wither::prelude::*;
use wither::bson::{doc, oid::ObjectId};
use wither::mongodb::{Client, error::Result};

// Now we define our model. Simple as deriving a few traits.
#[derive(Debug, Model, Serialize, Deserialize)]
#[model(index(keys=r#"doc!{"email": 1}"#, options=r#"doc!{"unique": true}"#))]
struct User {
    /// The ID of the model.
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    /// This field has a unique index on it.
    pub email: String,
}

fn main() -> Result<()> {
    // Connect & sync indexes.
    let db = Client::with_uri_str("mongodb://localhost:27017/")?.database("mydb");
    User::sync(db.clone())?;

    // Create a user.
    let mut me = User{id: None, email: String::from("my.email@example.com")};
    me.save(db.clone(), None)?;

    // Update user's email address.
    me.update(db.clone(), None, doc!{"$set": doc!{"email": "new.email@example.com"}}, None)?;

    // Fetch all users.
    for user in User::find(db.clone(), None, None)? {
        println!("{:?}", user);
    }
    Ok(())
}
