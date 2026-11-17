# Reference

- [Language manual](https://agentseedlanguage-cpu.github.io/agentseed/book/)
- [API reference](https://agentseedlanguage-cpu.github.io/agentseed/api/)
- Specification: `docs/book/ASL_V0.1.0_Spec.md` in the [repository](https://github.com/agentseedlanguage-cpu/agentseed)
- Architecture (arc42): `ASL_V_0_2_0_ARC42.md` in the repository

## Toolchain pieces

| Piece | Role |
| --- | --- |
| `seedc` | Compiler library: lexer, parser, semantic checks (names, types, effects, taint, contracts), lowering, binary writer |
| `seedvm` | Virtual machine that loads `.aslb` and runs it |
| `seedc-cli` | The `seed` command |
| `seedfmt`, `seedls`, `seeddbg`, `seedpkg` | Formatter, language server, debugger, package tool |
