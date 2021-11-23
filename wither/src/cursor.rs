use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::Stream;
use mongodb::Cursor;

use crate::error::{Result, WitherError};
use crate::Model;

/// A cursor of model documents.
pub struct ModelCursor<T> {
    cursor: Cursor,
    marker: std::marker::PhantomData<T>,
}

impl<T: Model> ModelCursor<T> {
    pub(crate) fn new(cursor: Cursor) -> Self {
        Self { cursor, marker: std::marker::PhantomData }
    }
}

// Impl Unpin on this container as we do not care about this container staying pinned,
// only the underlying `Cursor` needs to remain pinned while we poll from this vantage point.
impl<T> Unpin for ModelCursor<T> {}

impl<T: Model> Stream for ModelCursor<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let doc = match Pin::new(&mut self.cursor).poll_next(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(WitherError::from(err)))),
            Poll::Ready(Some(Ok(doc))) => doc,
        };
        match Model::instance_from_document(doc) {
            Ok(model) => Poll::Ready(Some(Ok(model))),
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}
