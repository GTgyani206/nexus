# Project Nexus

Nexus is a single-toolchain language stack that compiles VORTEX source to an interaction-net IR and executes it through the VICE reduction runtime.

## Install

```bash
npm install -g nexus-lang
```

```bash
curl -fsSL https://raw.githubusercontent.com/GTgyani206/nexus/main/install.sh | bash
```

```bash
wget -qO- https://raw.githubusercontent.com/GTgyani206/nexus/main/install.sh | bash
```

## Quick Start

```bash
echo "2 + 3" > example.vx
nexus run example.vx --ic
```

Expected output:

```text
5
```

## CLI

```bash
nexus run program.vx          # legacy interpreter
nexus run program.vx --ic     # IC pipeline (VORTEX -> IR -> VICE)
nexus run program.vx --ic --stats
nexus net program.vx          # dump generated IR net
nexus check program.vx        # parse-only check
nexus repl                    # interactive mode
```

## How It Works

VORTEX handles lexing, parsing, and AST construction from `.vx` source files.  
The `codegen_ic` pass lowers statements/expressions into `nexus-ir` graph cells (`Num`, `Op`, `Con`, etc.) with explicit port wiring.  
VICE exposes `reduce_ir` and `reduce_ir_with_stats` to consume that IR directly from memory.  
The `nexus run --ic` command executes this full bridge path without using the tree-walking interpreter.

## Benchmark

Use these commands to record the same benchmark format for releases:

```bash
time nexus run fib.vx
time nexus run fib.vx --ic
nexus run fib.vx --ic --stats
```

Stats output format:

```text
reductions: <N>, time_ms: <T>
```

## Week 1 Scope

- Workspace monorepo with `crates/vortex`, `crates/vice`, and `crates/nexus-ir`
- Shared IR crate with ports, cells, redexes, debug map, and alloc/connect helpers
- IC codegen pass wired behind `nexus run --ic`
- VICE IR reducer API with stats output
- npm + curl/wget install assets and release workflow

## Roadmap

See the versioned roadmap in [docs/roadmap.md](./docs/roadmap.md).
