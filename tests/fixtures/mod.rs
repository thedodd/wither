extern crate mongodb;
extern crate wither;

use mongodb::db::{Database, ThreadedDatabase};

pub fn clear_data_for_models(db: Database, models: Vec<wither::Model>) {
    for model in models {
        let coll = db.Collection(model::COLLECTION_NAME);
    }
}
