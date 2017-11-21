use bson::Document;
use chrono;
use mongodb::coll::Collection;
use mongodb::coll::options::UpdateOptions;
use mongodb::common::WriteConcern;
use mongodb::error::Result;


/// A trait definition of objects which can be used to manage schema migrations.
pub trait Migration {
    /// The function which is to execute this migration.
    fn execute<'c>(&self, coll: &'c Collection) -> Result<()>;
}

/// A migration type which allows execution until the specifed threshold date. Then will no-op.
///
/// This migration type works nicely in environments where multiple instances of the system — in
/// which this migration is defined — are continuously running, even during deployment cycles.
/// Highly available systems. With an `IntervalMigration`, each instance will execute the migration
/// at boottime, until the threshold date is passed. This will compensate for write-heavy
/// workloads, as the final instance to be updated will ensure that any documents, written by
/// previously running older versions of the system, will be properly migrated until all instances
/// have been updated. **As long as you ensure your migrations are idempotent — which you should
/// ALWAYS BE DOING — this will work quite nicely.**
pub struct IntervalMigration {
    /// The name for this migration. Must be unique per collection.
    pub name: String,

    /// The UTC datetime when this migration should no longer execute.
    ///
    /// Use something like: `chrono::Utc.ymd(2017, 11, 20).and_hms(22, 37, 34)`.
    pub threshold: chrono::DateTime<chrono::Utc>,

    /// The filter to be used for selecting the documents to update.
    pub filter: Document,

    /// The document to be used for the `$set` operation of the update.
    pub set: Document,

    /// The document to be used for the `$unset` operation of the update.
    pub unset: Document,
}

impl Migration for IntervalMigration {
    fn execute<'c>(&self, coll: &'c Collection) -> Result<()> {
        // If the migrations threshold has been passed, then no-op.
        if chrono::Utc::now() > self.threshold {
            return Ok(());
        };

        info!("Executing migration '{}' against '{}'.", &self.name, coll.namespace);
        let update = doc!{"$set": self.set.clone(), "$unset": self.unset.clone()};
        let options = UpdateOptions{upsert: Some(false), write_concern: Some(WriteConcern{w: 1, w_timeout: 0, j: true, fsync: false})};
        let res = coll.update_many(self.filter.clone(), update, Some(options))?;
        info!("Successfully executed migration '{}' against '{}'. {} matched. {} modified.", &self.name, coll.namespace, res.matched_count, res.modified_count);
        Ok(())
    }
}
