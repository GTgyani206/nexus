# Nexus Roadmap

## v0.1 (Week 1 Bridge)

- Monorepo workspace
- Shared `nexus-ir` crate
- VORTEX `codegen_ic` pass for arithmetic, let bindings, and function-call bridge
- VICE `reduce_ir` runtime entry point with reduction stats
- Unified `nexus` CLI (`run`, `check`, `net`, `repl`)
- npm + install script + release workflow scaffolding

## v0.2 (Parallel Reduction)

- Parallel reduction scheduling in VICE (`rayon`)
- Improved redex work stealing and batching
- Better IC net snapshots/clone paths for distributed execution
- Extended benchmark suite and perf dashboards

## v0.3 (Broader Language Coverage)

- Full control-flow lowering (`if/else`, loops, pattern-style branching)
- Better function lowering and multi-argument call optimization
- Improved type-checking and compiler diagnostics

## v0.4+

- Floating-point native cells
- Network/distributed reduction transport
- Runtime visualization and debugger integration
