# Verify it yourself

A VeriCrypt report is meant to be checked by someone who does not trust the person who made it.

1. Get the signer's **public** key file (`my-key.pub.json`) from them through a channel you trust.
2. Run `vericrypt verify report.json --pubkey my-key.pub.json`. A valid report prints `RESULT: VERIFIED`.
3. Edit any byte of `report.json` and run it again. Verification fails (exit code 1).
4. Try a different public key. Verification fails.
5. Remove the signature. The report fails unless you pass `--allow-unsigned`.

Pinning the key matters: without it you only learn that the report is consistent with the key inside it.

The CBOM can be checked separately against the CycloneDX 1.7 schema with `python3 scripts/validate_cbom.py out/cbom.json`.
