FROM rust:1-slim-buster
RUN apt-get update
RUN apt-get install build-essential git cmake pkg-config libssl-dev libudev-dev curl -y
RUN cargo install cargo-chef --locked
RUN cargo install --force cargo-make
RUN cargo install --force cargo-tarpaulin
RUN rustup component add rustfmt
RUN cargo install --force cargo2junit
RUN cargo install --locked cargo-deny
