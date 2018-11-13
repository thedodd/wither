contributing
============

### development
#### tests
For easy backend integration, this project is using docker compose to manage MongoDB instances. Before kicking off tests locally, simply execute `docker-compose up -d` to bring up all of the different MongoDB instances.

Now you just need to invoke the tests with the appropriate environment variables exposed:

```bash
# Test against MongoDB 3.2.
HOST=localhost PORT=27017 cargo test -p wither --tests --lib -- --test-threads=1

# Test against MongoDB 3.4.
HOST=localhost PORT=27117 cargo test -p wither --tests --lib -- --test-threads=1

# Test against MongoDB 3.6.
HOST=localhost PORT=27217 cargo test -p wither --tests --lib -- --test-threads=1

# Test against MongoDB 4.0.
HOST=localhost PORT=27317 cargo test -p wither --tests --lib -- --test-threads=1
```

For the compile tests, you will need to use nightly.

```bash
# Run the compile tests.
cargo +nightly test -p wither_derive --tests --lib
```
