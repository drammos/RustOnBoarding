# `bongo-mong`

A manager service for MongoDb connection pools.

## Usage

Insert this line under `[dependencies]` in your `Cargo.toml`

```toml
[dependencies]
...
bongo_mong = "0.3"
```

The library provides an implementation of the `dao` traits for common collections, exposed in the module `collections`.

This is only available when the `collections` feature is enabled:

```toml
[dependencies]
...
bongo_mong = { version = "0.3", features = ["collections"] }
```

## Development

### System requirements

* `rust` ([installation instructions][install-rust]).

### Clone the repository

```sh
$ git clone git@git.wappier.com:wappier/oxidised/bongo-mong.git
```

### Run examples

* `simple`

  This demonstrates the usage of `PoolManager`.

  ```sh
  cargo run --example simple
  ```

* `collection`

  This demonstrates usage of the library on a collection.

  > Requires running mongo server. To start run
  >
  > ```sh
  > $ docker run --name mongodb -p 27017:27017 -d mongo:4.0.5
  > ```

  ```sh
  cargo run --example collection --features collections
  ```

* `installations`

  This demonstrates usage of the library on the `installations` collection.

  > Requires running mongo server. To start run
  >
  > ```sh
  > $ docker run --name mongodb -p 27017:27017 -d mongo:4.0.5
  > ```

  ```sh
  cargo run --example installations --features collections
  ```
### Run tests

```sh
$ cargo test --all-features
```

[install-rust]: https://www.rust-lang.org/tools/install
