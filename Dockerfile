FROM rust:1.25

LABEL maintainer="Anthony Josiah Dodd <Dodd.AnthonyJosiah@gmail.com>"

# Install watcher extension.
RUN cargo install cargo-watch

# Copy over needed files.
WORKDIR /wither
COPY ./Cargo.lock Cargo.lock
COPY ./Cargo.toml Cargo.toml
COPY ./src src
COPY ./tests tests

RUN cargo build

# Use a CMD here (instead of ENTRYPOINT) for easy overwrite in docker ecosystem.
CMD ["cargo", "test", "--lib", "--tests"]
