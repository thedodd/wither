use mongodb::Cursor;
use mongodb::error::Result;

use crate::Model;

/// A cursor of model documents.
pub struct ModelCursor<T> {
    cursor: Cursor,
    marker: std::marker::PhantomData<T>,
}

impl<T: Model> ModelCursor<T> {
    pub(crate) fn new(cursor: Cursor) -> Self {
        Self{cursor, marker: std::marker::PhantomData}
    }
}

impl<T: Model> Iterator for ModelCursor<T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let doc = match self.cursor.next() {
            None => return None,
            Some(Err(err)) => return Some(Err(err)),
            Some(Ok(doc)) => doc,
        };
        match Model::instance_from_document(doc) {
            Ok(model) => Some(Ok(model)),
            Err(err) => Some(Err(err)),
        }
    }
}
