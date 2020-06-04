contributing
============
This project is currently setup as three different crates.

- `wither`: the main business logic crate.
- `wither_derive`: the custom derive logic.

### publishing
To publish a new version of `wither` or `wither_derive`, execute the following command: `cargo publish --manifest-path $TARGET_DIR/Cargo.toml`, where `$TARGET_DIR` is the directory of the crate which is to be released.

Keep in mind that we need to keep the major and minor versions of these two crates in sync. Else, it could cause some serious confusion.

**Don't forget to tag the release in git.**

### development
#### tests
For easy backend integration, this project is using docker compose to manage MongoDB instances. Before kicking off tests locally, simply execute `docker-compose up -d` to bring up all of the different MongoDB instances.

Now you just need to invoke the tests with the appropriate environment variables exposed:

```bash
# Test against MongoDB 3.6.
HOST=localhost PORT=27217 cargo test -p wither --tests --lib -- --test-threads=1

# Test against MongoDB 4.0.
HOST=localhost PORT=27117 cargo test -p wither --tests --lib -- --test-threads=1

# Test against MongoDB 4.2.
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
