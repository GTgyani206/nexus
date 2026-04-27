# HVM Engine prototype ---  Renamed as VICE

## File Structure
```
VICE/
├── Cargo.toml
├── README.md
├── examples/
│   ├── factorial.hvm
│   ├── fibonacci.hvm
│   └── list_operations.hvm
├── src/
│   ├── main.rs          # CLI entry point
│   ├── net.rs           # Graph implementation
│   ├── parser.rs        # Simple parser for .hvm files
│   ├── interaction.rs   # Core interaction rules
│   ├── runtime.rs       # Evaluation engine
│   ├── visualizer.rs    # Optional graph visualization
│   └── lib.rs           # Library exports
└── tests/
    └── integration_tests.rs
```

## Understanding the Codebase

1. The Graph Network(net.rs) - The Engine Block :
  This is the fundamental physical structure of the engine
2. Interaction Rules(interaction.rs) - The Combustion System :
  The rules that determine how energy flows through the system
3. Runtime Engine(runtime.rs) - The Control Unit :
  The engine that manages the engine's list_operations
4. Parser (parser.rs) - The Fuel Injection System :
  Converts the human instruction into a form the engine can use
5. Visualizer(visualizer.rs) - The Diagonistic System :
  Shows what's happening inside the engine.

## The Key Innovation: Automatic Parallelism
### Traditional Engines vs. VICE

#### Traditional Engine (Normal Languages):
  Parts move in a fixed sequence
  One piston fires, then the next, then the next
  To get more power, a skilled mechanic must manually reconfigure the engine
  Making it use 8 cylinders at once requires complex custom engineering

#### VICE Engine (Interaction Combinator Runtime):
  Parts move whenever they're ready to move
  If two pistons can fire at the same time, they will
  The engine automatically uses all available cylinders
  No special engineering required - parallelism is built into the physics

> This is why VICE (like HVM) is revolutionary - it automatically parallelizes computation without the programmer having to think about threads, locks, race conditions, or any of the complexity that normally comes with parallel programming.

## Use of VICE in VORTEX

VORTEX language will be the sleek car built around this revolutionary engine:

  -  Programmers write code in VORTEX syntax (driving the car)
  -  The compiler translates this into VICE's interaction net format (the car's controls connect to the engine)
  - VICE automatically executes this with maximum parallelism (the engine runs at full power)
  -  Results are translated back to VORTEX's format (the speedometer shows the performance)

This gives programmers incredible power - they get all the benefits of parallelism without having to be expert engine mechanics!
