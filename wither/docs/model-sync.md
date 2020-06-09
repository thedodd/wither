### sync
[`Model::sync`](./trait.Model.html#method.sync) will synchronize your model's indexes with the database. It is an integral component of this system & allows you to delegate a majority of your database administration tasks to your services which are actually using the database.

This routine should be called once per model, early on at boottime. This routine will destroy
any indexes found on this model's collection which are not defined on this model (barring the
default index on `_id`).

**PLEASE NOTE:** as of the `0.9.0-alpha.0` release, corresponding to the mongodb `1.0` release, index management has not yet been implemented in the mongodb driver, and thus the index syncing features of `Model::sync` have been temporarily disabled. The hope is that the mongodb team will be able to land their index management code in the driver soon, at which point we will re-enable the `Model::sync` functionality.

If this is important to you, please head over to [wither#51](https://github.com/thedodd/wither/issues/51) and let us know!
