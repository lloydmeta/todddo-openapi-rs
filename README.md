## todddo-openapi-rs [![Build Status](https://travis-ci.org/lloydmeta/todddo-openapi-rs.svg?branch=master)](https://travis-ci.org/lloydmeta/todddo-openapi-rs) [![codecov](https://codecov.io/gh/lloydmeta/todddo-openapi-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/lloydmeta/todddo-openapi-rs)

This project explores using using Rust in a DDD-esque fashion (cleanly separated `domain` and `infra`), along with
the current (as of writing) Rust web tools, such as `actix-web`, and generated-from-source OpenAPI spec.

In specific:

- [actix-web](https://actix.rs/)
- Mixing Future 0.1 with async/await-ready Future 0.3
- Using `async/await` via nightly
  - Using it with traits via [`async-trait`](https://github.com/dtolnay/async-trait)
- Using paperclip to generate [OpenAPI](https://paperclip.waffles.space/paperclip/) from source
- Using [`future_locks`](https://docs.rs/futures-locks/0.3.3/futures_locks/) for async mutexes
- Compiling static assets into the binary.
- DDD-esque project structuring using workspaces to keep dependencies pure
- Postponing of concrete types for interfaces (`trait`s) to maximise testability
- Enabling code coverage in Rust w/ workspaces (credit to Ana Gelez's [article](https://blog.funkwhale.audio/~/Rust@baptiste.gelez.xyz/rust-nightly-travis-ci-and-code-coverage)) 

This is spiritually the sister project to [the Go version](https://github.com/lloydmeta/todddo-openapi).

## Running

This project requires nightly, so you'll need to `rustup toolchain install nightly`, then run via

```shell
cargo +nightly run
``` 

If, for some reason, nightly is borked, `nightly-2019-08-20-x86_64-apple-darwin` has been known to work; just install
the right toolchain (`nightly-2019-08-20-${your-architecture}`) and run with that instead.