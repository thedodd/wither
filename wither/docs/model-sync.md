### sync
[`Model::sync`](./trait.Model.html#method.sync) will synchronize your model's indexes with the database. It is an integral component of this system & allows you to delegate a majority of your database administration tasks to your services which are actually using the database.

This routine should be called once per model, early on at boot-time. This routine will destroy any indexes found on this model's collection which are not defined on this model (barring the default index on `_id`).
