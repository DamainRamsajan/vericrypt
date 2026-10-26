# Install and run VeriCrypt v2

Three steps: **get the tools → build → try it.** About 15 minutes the first time, mostly compiling.

## Step 0 — What you need (once)

| You need | Ubuntu / Debian / WSL2 | macOS |
|---|---|---|
| C compiler, pkg-config, OpenSSL headers, git, unzip | `sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev clang git curl unzip` | `xcode-select --install` then `brew install openssl pkg-config` |
| Rust (current stable) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` then open a new terminal | same command |

Check: `cargo --version` prints a version (I tested 1.91). Windows: use **WSL2 with Ubuntu** and follow the Ubuntu column; native Windows is untested.

## Step 1 — Get the code (pick ONE route)

**Route A: the zip (easiest, recommended).**
```bash
unzip vericrypt-v2-source.zip
cd vericrypt-v2
```

**Route B: the patch, on top of your own clone.**
```bash
git clone https://github.com/DamainRamsajan/vericrypt.git
cd vericrypt
git apply /path/to/vericrypt-v2.patch
```
Both routes give byte-identical source (I compared them).

## Step 2 — Build

```bash
cargo build --release -p vericrypt
```
- First build: roughly 5–15 minutes depending on your CPU cores (it compiles about 300 dependencies). Later builds take seconds.
- The program lands at `./target/release/vericrypt`.
- Commercial edition (gated features need a license): add `--features license-gate`.

**Do not run `cargo update`.** The lock file pins one dependency (`signature 2.3.0-pre.4`) because the newer pre-release breaks the signing library.

## Step 3 — Try it (2 minutes, no OpenSSL knowledge needed)

The bundle includes test certificates. Run these from the project folder:

```bash
# 1. make a post-quantum signing key (SLH-DSA, FIPS 205)
./target/release/vericrypt keygen --out my-key.json

# 2. scan the sample certificates and sign the report
./target/release/vericrypt analyze \
  -i crates/vericrypt/tests/fixtures/pki \
  --out out \
  --public-issuers "Acme Public Test CA" \
  --sign-key my-key.json

# 3. verify it, pinning your public key
./target/release/vericrypt verify out/report.json --pubkey my-key.pub.json
```

You should see a summary like `Assets: 7 … Findings: 1 critical, 11 high, 3 medium`, then `RESULT: VERIFIED`.

Output files in `out/`:
- `report.json` — the signed evidence (what an auditor verifies)
- `cbom.json` — CycloneDX 1.7 cryptographic bill of materials
- `report.html` — open in a browser; print to PDF to share

## Run the tests (optional, ~1-2 minutes after the build)

```bash
cargo test -p vericrypt --lib engine::
cargo test -p vericrypt --test engine_pipeline --test engine_cli
```
Expect 19 passing tests. `python3 scripts/validate_cbom.py out/cbom.json` also checks the CBOM against the official CycloneDX schema (needs `pip install jsonschema referencing`).

## Use it on your own systems

```bash
# scan a folder of certificates/keys (repeat -i for more folders)
./target/release/vericrypt analyze -i /path/to/certs --out ./out \
  --public-issuers "Let's Encrypt,DigiCert" --private-issuers "My Internal CA" \
  --sign-key my-key.json

# probe live TLS servers (leaf certificate only in v2)
./target/release/vericrypt analyze --tls example.com:443 --out ./out

# compare two scans (what changed?)
./target/release/vericrypt diff old/report.json new/report.json --fail-on high

# check someone else's report
./target/release/vericrypt verify their-report.json --pubkey their.pub.json
```
Only probe servers you are authorised to test. Keep `my-key.json` private (it is created with permission 0600); share only `my-key.pub.json`.

Exit codes: `0` ok · `1` verification failed · `2` bad input · `3` findings at/above `--fail-on` · `4` license needed.

## If something goes wrong

| Symptom | Fix |
|---|---|
| `linker 'cc' not found` | install the build tools (Step 0) |
| `Could not find directory of OpenSSL installation` | install `libssl-dev` + `pkg-config` (Linux) or `export OPENSSL_DIR=$(brew --prefix openssl)` (macOS) |
| Compile errors mentioning `signature` or `slh_dsa` | you ran `cargo update`; restore `Cargo.lock` from the zip/patch |
| Rust version errors | `rustup update stable` |
| `git apply` fails | you are not on the original commit; use Route A |
| `vericrypt scan` prints a warning | that is the old v1 pipeline; use `analyze` |
| Build is very slow | normal on 1–2 cores; add `-j 4` if you have more |

## What I verified, and what I did not

Verified on a clean copy (Ubuntu 24.04, Rust 1.91): unzip → `cargo build --release` → the three quickstart commands → `RESULT: VERIFIED`; the patch route yields identical source; the engine test suite passes; in the quickstart, analysis plus signing took about 1 second in the release build.

Not tested: macOS, native Windows, ARM, installing Rust through rustup (my sandbox used Ubuntu's Rust packages), and whether `clang` is strictly required (I installed it in my test environment; it is harmless).
