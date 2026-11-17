# User manual

VeriChain records what autonomous agents are authorized to do and what they do, in a witnessed, append-only log that anyone can check offline. This page shows the quickest way to see that working.

> **Status.** VeriChain is pre-release. The specification is frozen and the core log and verifier crates are built and tested. There is no packaged download yet; you build from source.

## Try it in two minutes

You need the source and Rust (see Installation). Then:

```
cargo build --release -p vcd -p vc-verify
./target/release/vcd demo   ~/vc-demo     # makes a signed evidence pack from two demo agents
./target/release/vcd serve  ~/vc-demo     # dashboard at http://127.0.0.1:8787
./target/release/vcd verify ~/vc-demo     # command line check; exit 0 means every check passed
```

In the dashboard, press the tamper buttons. Editing a past statement or hiding the latest activity turns checks red.

## Verify a pack yourself

```
./target/release/vcv verify ~/vc-demo/pack.json \
  --trust ~/vc-demo/trust.json --policy ~/vc-demo/policy.json --require-seals
```

Exit code 0 means every check passed. See Verify for what each check means and why the trust file matters.

## Core ideas

| Idea | Meaning |
| --- | --- |
| Statement | A signed record of something an agent was authorized to do or did |
| Receipt | Proof that a statement is in the log |
| Evidence pack | Statements, receipts and checkpoints bundled so someone can verify them offline |
| Completeness C0 to C3 | How sure you can be that nothing was left out |
| Assurance A0 to A3 | How strongly a record is backed, from signed (A0) up to proof (A3) |
