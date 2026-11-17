# Verify it yourself

OMC's evidence is meant to be re-run without the vendor.

1. Clone the [repository](https://github.com/DamainRamsajan/Operating-Margin-Controller-) and run `python scripts/make_vectors.py`. This writes the public test vectors.
2. Run `python scripts/demo.py`. It produces signed Evidence Statements and checks the hash chains.
3. The standalone verifier lives in `omc_verify/` and depends only on `cbor2` and `cryptography`, so it can be audited on its own. Run it on the statements and vectors.
4. Recompute a published figure from its signed inputs and compare.

Today statements are signed with Ed25519 only. Post-quantum signing and transparency-service registration are on the backlog.
