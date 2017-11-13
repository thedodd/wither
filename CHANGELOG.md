changelog
=========

## 0.4
- adds `Model.update`.

## 0.3
- adds `Model::count`.

## 0.2
**NOTE:** is backwards incompatible with `0.1.0`.
- adopts usage of `associated constants` for `Model`. Thus, `Model::collection_name()` has been replaced by the associated constant `Model::COLLECTION_NAME` and is required when adopting the `Model` trait for your types.
- adds an actual implementation for `Model::find`.

**0.2.1:**
- no code changes here, just updating some docs & badge links.

## 0.1
Initial release.
