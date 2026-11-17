# Installation

## Build from source

You need Rust 1.85 or newer.

```
git clone https://github.com/agentseedlanguage-cpu/agentseed
cd agentseed
cargo build --release -p seedc-cli
./target/release/seed --version
```

The `seed` binary wraps the compiler and the VM. Add `target/release` to your `PATH`, or copy `seed` somewhere that is.

## Check it works

```
./target/release/seed check examples/hello.seed
./target/release/seed run examples/hello.seed
```

## Packaged installs

The language manual describes an npm package and an install script. The package name and script location are being confirmed, so this page does not list a command for them yet.
