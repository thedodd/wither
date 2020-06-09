use mongodb::sync::Cursor;

use crate::error::{Result, WitherError};
use crate::sync::ModelSync;

/// A cursor of model documents.
pub struct ModelCursorSync<T> {
    cursor: Cursor,
    marker: std::marker::PhantomData<T>,
}

impl<T: ModelSync> ModelCursorSync<T> {
    pub(crate) fn new(cursor: Cursor) -> Self {
        Self{cursor, marker: std::marker::PhantomData}
    }
}

impl<T: ModelSync> Iterator for ModelCursorSync<T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let doc = match self.cursor.next() {
            None => return None,
            Some(Err(err)) => return Some(Err(WitherError::from(err))),
            Some(Ok(doc)) => doc,
        };
        match ModelSync::instance_from_document(doc) {
            Ok(model) => Some(Ok(model)),
            Err(err) => Some(Err(err)),
        }
    }
}
