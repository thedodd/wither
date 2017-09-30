changelog
=========

### 0.2.0
**NOTE:** is backwards incompatible with `0.1.0`.
- adopts usage of `associated constants` for `Model`. Thus, `Model::collection_name()` has been replaced by the associated constant `Model::COLLECTION_NAME` and is required when adopting the `Model` trait for your types. 
- adds an actual implementation for `Model::find`.

### 0.1.0
Initial release.
