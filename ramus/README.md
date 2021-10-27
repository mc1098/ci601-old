
# Ramus

Ramus is a the project code which will be a RFC compliant HTTP server written as library for
Rust.

## Requirements

This is a Rust library so requires `rustc` in order to build `Ramus`, you can get `rustc` and more
using [`Rustup`, which is a tool for installing and version management of Rust](https://www.rust-lang.org/learn/get-started).

Ramus is built using the latest stable release of Rust and since 21/10/2021 this means that it is
built using the 2021 edition too.

[Cargo](https://doc.rust-lang.org/cargo/index.html) is the Rust package manager and the `cargo`
command is normally used over interacting with `rustc` directly.

## Building, Testing, Docs

You can build Ramus with:

```bash
cargo build 
```

You can run the unit and documentation tests with:

```bash
cargo test --all
```

Once you have built Ramus you can open the docs locally with:

```bash
cargo doc --open 
```


## Examples

TODO: Add examples to show off how `Ramus` can be used as a library in order to create a HTTP server
application.


