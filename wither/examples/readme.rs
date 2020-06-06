use futures::stream::StreamExt;
use serde::{Serialize, Deserialize};
use wither::prelude::*;
use wither::bson::{doc, oid::ObjectId};
use wither::mongodb::Client;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect & sync indexes.
    let db = Client::with_uri_str("mongodb://localhost:27417/").await?.database("mydb");
    User::sync(db.clone()).await?;

    // Create a user.
    let mut me = User{id: None, email: String::from("my.email@example.com")};
    me.save(db.clone(), None).await?;

    // Update user's email address.
    me.update(db.clone(), None, doc!{"$set": doc!{"email": "new.email@example.com"}}, None).await?;

    // Fetch all users.
    let mut cursor = User::find(db.clone(), None, None).await?;
    while let Some(user) = cursor.next().await {
        println!("{:?}", user);
    }
    Ok(())
}
