# Verify it yourself

AgentSeed's central claim is that its safety rules are checked by the compiler. You can test that on your own machine.

1. Get the toolchain from the [repository](https://github.com/agentseedlanguage-cpu/agentseed) (Rust compiler and VM, MIT license).
2. Run `seed check hello.seed` on a program. It type-checks and verifies without running anything.
3. Run `seed emit-ir hello.seed` to see what the compiler produced.
4. Run the conformance tests in the repository with `cargo test` and compare with the published results.

Guarantees apply to programs written in ASL, not to existing Python or JavaScript code.
