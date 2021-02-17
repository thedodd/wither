contributing
============
This project is currently composed of the following crates:

- `wither`: the main business logic crate.
- `wither_derive`: the custom derive logic.

### publishing
Just publish a tag for the corresponding crate to publish: `wither-v*` for `wither` and `wither-derive-v*` for `wither-derive`.

### development
#### tests
For easy backend integration, this project is using docker compose to manage MongoDB instances. Before kicking off tests locally, simply execute `docker-compose up -d` to bring up the latest version of MongoDB for testing. Modify the file if older versions are needed.

Now you just need to invoke the tests with the appropriate environment variables exposed:

```bash
# Execute tests & point to the mongo instance via env vars.
HOST=localhost PORT=27017 cargo test -p wither --tests --lib -- --test-threads=1
```

Execute the compile tests.

```bash
cargo test -p wither_derive --tests --lib
```

For doc tests, you will need to use nightly.

```bash
cargo +nightly test --features docinclude --doc
```
