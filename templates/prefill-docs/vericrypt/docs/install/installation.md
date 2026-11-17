# Installation

Three steps: get the tools, build, try it. About 15 minutes the first time, mostly compiling.

## 1. What you need (once)

| You need | Ubuntu, Debian or WSL2 | macOS |
| --- | --- | --- |
| Compiler and libraries | `sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev clang git curl unzip` | `xcode-select --install` then `brew install openssl pkg-config` |
| Rust (current stable) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | same |

On Windows use WSL2 with Ubuntu. Native Windows, macOS and ARM are untested.

## 2. Get the code and build

```
git clone https://github.com/DamainRamsajan/vericrypt.git
cd vericrypt
cargo build --release -p vericrypt
```

The program lands at `./target/release/vericrypt`. The first build compiles about 300 dependencies (5 to 15 minutes). **Do not run `cargo update`**: the lock file pins one dependency (`signature 2.3.0-pre.4`) because newer pre-releases break the signing library.

## 3. Test (optional)

```
cargo test -p vericrypt --lib engine::
cargo test -p vericrypt --test engine_pipeline --test engine_cli
```

Expect 19 passing tests.

## If something goes wrong

| Symptom | Fix |
| --- | --- |
| `linker 'cc' not found` | Install the build tools (step 1) |
| OpenSSL directory not found | Install `libssl-dev` and `pkg-config` (Linux) or `export OPENSSL_DIR=$(brew --prefix openssl)` (macOS) |
| Errors mentioning `signature` or `slh_dsa` | You ran `cargo update`; restore `Cargo.lock` |
| Rust version errors | `rustup update stable` |
| `vericrypt scan` prints a warning | That is the old v1 command. Use `analyze` |
