# Sending VeriCrypt to a pilot customer (licence keys)

Everything here was tested between 30 Oct and 1 Nov 2026 except where marked.

## One-time setup
1. **Licence signing key.** Either
   - **use your existing key:** the public half is `paseto_public_key.hex` in the repo root. Its private half should be your Supabase secret `PASETO_PRIVATE_KEY`. Put that value in a file and check it matches: `vericrypt-license issue --private-key FILE --public-key-file paseto_public_key.hex ...` refuses to issue if it does not match; or
   - **make a new pair:** `cargo run -p vericrypt --features issuer --bin vericrypt-license -- keygen --out ~/vericrypt-keys`, copy `paseto_public_key.hex` into the repo root and commit it (public key only), and put the private key into the Supabase secret. Keep the private key out of git.
2. Build the issuing tool once: `cargo build --release -p vericrypt --features issuer --bin vericrypt-license`. **Never send this tool to customers.**

## For each pilot customer
1. Build the customer binary (the public key is embedded automatically from the repo root):
   `cargo build --release -p vericrypt --features license-gate`
   The build now stops with an error if the public key is missing, instead of silently producing a binary no licence can activate.
2. Issue a licence:
   `vericrypt-license issue --private-key ~/vericrypt-keys/paseto_private_key.hex --public-key-file paseto_public_key.hex --org "Customer Bank" --email ciso@customer.com --days 90`
   It prints one long token starting `v4.public.`
3. Send the customer `target/release/vericrypt` and the token.
4. The customer runs once: `vericrypt activate --key "<token>"`. After that signing, the HTML report and `diff` work. Everything else always works without a licence, and `verify` is always free.

## Package and sign a release (new)
`bash scripts/make_release.sh /path/to/your-signing-key.json` builds the licence-gated binary, writes `dist/` with the binary, `SHA256SUMS`, a CycloneDX software bill of materials (`vericrypt.sbom.cdx.json`), a cryptographic bill of materials of the binary itself (`vericrypt.cbom.cdx.json`: which algorithms and protocols it uses and why) and a post-quantum signature file (`*.sig.json`) for each. Use the same key you created with `vericrypt keygen`; keep it private and share only the matching `.pub.json`.
Give the customer the `dist/` files plus your public key file. They check the binary offline, before running it:
`vericrypt verify-file vericrypt --pubkey your-key.pub.json` (expect `RESULT: VERIFIED`) and `sha256sum -c SHA256SUMS`.
Banks' vendor reviews ask for this kind of evidence (SBOM, signed releases). Tested on 1 Nov 2026 with a debug build.

## Before you send anything
`bash scripts/verify_licence_e2e.sh` must print `ALL LICENCE CHECKS PASSED`, and `cargo test -p vericrypt` must pass (31 tests).

## Limits you should know
- Licences can expire but cannot be revoked remotely (the tool works offline).
- `max_assets` is written into the token but not enforced yet.
- The Supabase function was rewritten to the correct token format and the signing code was tested in Node, not in Deno. Deploy it and issue one test licence before relying on it. The offline `vericrypt-license` tool does not depend on Supabase.
- The binary no longer needs OpenSSL (checked on a debug build: it links only to the C library). A fully static build can be made with: `sudo apt-get install -y musl-tools && rustup target add x86_64-unknown-linux-musl && cargo build --release --target x86_64-unknown-linux-musl -p vericrypt --features license-gate`. **I could not test that build from my sandbox.**
- Tell the customer what the licence includes: only signed reports, the HTML report and drift diff. Do not promise TEE, VeriChain or custom-axiom features; they do not exist.
