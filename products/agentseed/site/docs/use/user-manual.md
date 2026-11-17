# User manual

AgentSeed is the open-source toolchain for ASL, a language for autonomous agents. The compiler `seedc` turns `.seed` source into `.aslb` bytecode, and the virtual machine `seedvm` runs it deterministically. The `seed` command wraps both.

## Your first program

Save this as `hello.seed`:

```
agent hello {
    fn main() -> i32 {
        print("Hello, Agent!");
        0
    }
}
```

Then:

```
seed check hello.seed    # type-check and verify, no output file
seed run hello.seed      # compile and run
seed build hello.seed    # write hello.aslb
```

## Commands

| Command | What it does |
| --- | --- |
| `seed build <file>` | Compile a `.seed` file to an `.aslb` bytecode module |
| `seed check <file>` | Type-check and verify without emitting bytecode |
| `seed run <file>` | Compile and run on the VM (`--seed N` sets the deterministic random seed) |
| `seed emit-ir <file>` | Print the intermediate representation (text, binary or Graphviz DOT) |
| `seed emit-grammar` | Export the GBNF grammar used for constrained model decoding |
| `seed prove <file>` | Generate an execution proof from a program (static analysis only) |

## Strata

ASL has four grammar strata. S0 (Seed) is for beginners and sandboxed agents, S1 (Core) is the production default, S2 (Full) adds self-evolution and reinforcement-learning hooks, and S3 (System) is for trusted orchestrators and requires human sign-off.

For the full language, read the [language manual](https://agentseedlanguage-cpu.github.io/agentseed/book/).
