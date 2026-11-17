# User manual

VeriCrypt scans certificates and keys, classifies their quantum exposure, and writes a signed report that someone else can verify.

> **Status.** The v2 evidence engine is built and tested. You build it from source. Everything on this page is covered by automated tests or was checked by hand on a clean build.

## Quickstart (about 2 minutes after the build)

```
# 1. make a post-quantum signing key (SLH-DSA, FIPS 205)
./target/release/vericrypt keygen --out my-key.json

# 2. scan the sample certificates and sign the report
./target/release/vericrypt analyze -i crates/vericrypt/tests/fixtures/pki --out out \
  --public-issuers "Acme Public Test CA" --sign-key my-key.json

# 3. verify it, pinning your public key
./target/release/vericrypt verify out/report.json --pubkey my-key.pub.json
```

You should see a summary, then `RESULT: VERIFIED`. Output files in `out/`:

| File | What it is |
| --- | --- |
| `report.json` | The signed evidence an auditor verifies |
| `cbom.json` | CycloneDX 1.7 cryptographic bill of materials |
| `report.html` | Open in a browser; print to PDF to share |

## Use it on your own systems

```
vericrypt analyze -i /path/to/certs --out ./out \
  --public-issuers "Let's Encrypt,DigiCert" --private-issuers "My Internal CA" --sign-key my-key.json
vericrypt analyze --tls example.com:443 --out ./out       # probe a live server (leaf certificate only)
vericrypt diff old/report.json new/report.json --fail-on high
vericrypt verify their-report.json --pubkey their.pub.json
```

Only probe servers you are authorised to test. Keep `my-key.json` private (created with permission 0600) and share only `my-key.pub.json`.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | OK |
| 1 | Verification failed |
| 2 | Bad input |
| 3 | Findings at or above `--fail-on` |
| 4 | License needed (builds with `license-gate`) |
