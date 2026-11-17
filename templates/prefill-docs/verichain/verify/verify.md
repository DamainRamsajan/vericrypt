# Verify it yourself

VeriChain's point is that you do not have to trust the operator. You can check an evidence pack on your own machine.

1. Build the verifier from the [repository](https://github.com/intellica-ai-llc/verichain) (`cargo build --release -p vc-verify`).
2. Get an evidence pack. For a first try, create one with `vcd demo ~/vc-demo`.
3. Run `vcv verify pack.json --trust trust.json --policy policy.json --require-seals`.
4. Change one byte of the pack and run it again. The check that covers that field fails.

## Why you pass your own trust file

A pack carries its own trust anchors and policy, which says nothing about who *you* trust. Without a trust file the report only shows that the pack is consistent with itself. Pass your own trust and policy to hold the pack to the minimum completeness, assurance and witness count you require.

## What is not yet verified

The verifier is built to compile to WebAssembly, but that build has not been verified, and an independent adversarial review is still pending. These are listed in the repository's `vc-verify` README.
