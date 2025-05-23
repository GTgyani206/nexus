# Vortex Language 🌪️

Vortex is a modern, GPU-accelerated programming language designed for parallel computation and high-performance computing. It features both traditional CPU execution and innovative GPU-accelerated constructs with an interactive REPL environment.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/LICENSE-GNU%20GPL%20v3.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-green.svg)](https://github.com)

## 🚀 Features

- **Dual-Mode Execution**: Traditional CPU and GPU-accelerated parallel processing
- **Interactive REPL**: Full-featured Read-Eval-Print Loop with command history
- **Smart Syntax**: Clean, Python-inspired syntax with GPU-specific constructs
- **Type System**: Optional static typing with type inference
- **Function Support**: First-class functions with CPU/GPU variants
- **Memory Safe**: Built with Rust for memory safety and performance
- **Cross-Platform**: Runs on Windows, macOS, and Linux

## 📦 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/vortex-lang.git
cd vortex-lang

# Build the project
cargo build --release

# Run the REPL
cargo run
```

### Your First Vortex Program

Create a file called `hello.vx`:

```vortex
// Variable declarations
let name = "World"
let count = 42

// Print statements
print("Hello")
print(name)
print("Count:")
print(count)

// Function definition
fn greet(person: String) -> String:
    return "Hello " + person

// Function call
let greeting = greet("Vortex")
print(greeting)
```

Run it:

```bash
cargo run hello.vx
```

## 🎮 Interactive REPL

Start the interactive REPL by running without arguments:

```bash
cargo run
```

### REPL Features

- **📚 Command History**: Navigate with ↑/↓ arrows
- **🎨 Colored Output**: Syntax highlighting and error messages
- **📝 Multi-line Input**: Automatic detection for functions and loops
- **💾 File Loading**: Load and execute external files
- **🔄 State Management**: Reset and inspect interpreter state

### REPL Commands

| Command | Description |
|---------|-------------|
| `:help` | Show help message |
| `:exit` | Exit the REPL |
| `:clear` | Clear the screen |
| `:history` | Show command history |
| `:load <file>` | Load and execute a file |
| `:env` | Show environment variables |
| `:reset` | Reset interpreter state |

### Example REPL Session

```vortex
vortex> let x = 42
vortex> print(x)
Output: 42

vortex> fn square(n: Int) -> Int:
...     return n * n
...
vortex> let result = square(7)
vortex> print(result)
Output: 49

vortex> :exit
Goodbye!
```

## 📖 Language Syntax

### Variables

```vortex
// Immutable variables
let x: Int = 42
let name: String = "Vortex"

// Mutable variables
let mut counter: Int = 0
let mut temperature: Float = 98.6
```

### Control Flow

#### Traditional Conditionals
```vortex
if x > 10:
    print("x is large")
then x == 5:
    print("x is five")
else:
    print("x is small")
```

#### GPU-Style Branching
```vortex
branch x > 10 => print("large")
branch x == 5 => print("five")
fallback => print("small")
```

### Loops

#### Standard Loops
```vortex
// Range function syntax
for i in range(0, 10):
    print(i)

// Range operator syntax
for i in 0..10:
    print(i)
```

#### Parallel Loops (GPU-Accelerated)
```vortex
parallel i in 0..1000:
    // Executed in parallel on GPU
    data[i] = data[i] * 2
```

### Functions

#### CPU Functions
```vortex
fn add(a: Int, b: Int) -> Int:
    return a + b

fn fibonacci(n: Int) -> Int:
    if n <= 1:
        return n
    else:
        return fibonacci(n - 1) + fibonacci(n - 2)
```

#### GPU Functions
```vortex
@gpu fn vector_add(a: Float, b: Float) -> Float:
    return a + b

@gpu fn matrix_multiply(a: Matrix, b: Matrix) -> Matrix:
    parallel i in 0..a.rows:
        parallel j in 0..b.cols:
            result[i][j] = dot_product(a.row(i), b.col(j))
    return result
```

### Data Types

| Type | Description | Example |
|------|-------------|---------|
| `Int` | 64-bit integer | `42` |
| `Float` | 64-bit floating point | `3.14159` |
| `String` | UTF-8 string | `"Hello World"` |
| `Boolean` | Boolean value | `true`, `false` |
| `Array` | Dynamic array | `[1, 2, 3, 4]` |

## 🖥️ Usage Modes

### Interactive Mode
```bash
cargo run
# Starts the REPL
```

### File Execution Mode
```bash
cargo run program.vx
# Executes the specified file
```

### Help
```bash
cargo run -- --help
# Shows usage information
```

## 🔧 GPU Acceleration

Vortex features simulated GPU acceleration for development and testing:

- **Parallel Loops**: Automatic parallelization with `parallel` keyword
- **GPU Functions**: Functions marked with `@gpu` run on simulated GPU
- **Memory Management**: Automatic data transfer simulation
- **Performance Monitoring**: Built-in GPU execution logging

### GPU Example

```vortex
// Define GPU-accelerated function
@gpu fn parallel_sum(arr: Array) -> Float:
    let sum = 0.0
    parallel i in 0..arr.length:
        sum += arr[i]
    return sum

// Execute with GPU acceleration
let numbers = [1.0, 2.0, 3.0, 4.0, 5.0]
let result = parallel_sum(numbers)
print("Sum:")
print(result)
```

## 📁 Project Structure

```
vortex-lang/
├── src/
│   ├── main.rs          # Entry point and CLI handling
│   ├── lexer.rs         # Tokenization
│   ├── parser.rs        # Syntax analysis
│   ├── ast.rs           # Abstract Syntax Tree
│   ├── interpreter.rs   # Code execution
│   ├── gpu_runtime.rs   # GPU simulation
│   └── repl.rs          # Interactive REPL
├── examples/
│   ├── example.vx       # Comprehensive example
│   └── test_range.vx    # Range syntax tests
├── Cargo.toml           # Rust dependencies
└── README.md            # This file
```

## 🛠️ Development

### Prerequisites

- Rust 1.70+ with Cargo
- Git

### Building from Source

```bash
# Clone repository
git clone https://github.com/your-org/vortex-lang.git
cd vortex-lang

# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run
```

### Dependencies

- `rustyline`: Command history and line editing
- `colored`: Terminal color output

## 📚 Examples

### Hello World
```vortex
print("Hello, Vortex!")
```

### Fibonacci Sequence
```vortex
fn fibonacci(n: Int) -> Int:
    if n <= 1:
        return n
    else:
        return fibonacci(n - 1) + fibonacci(n - 2)

for i in 0..10:
    print("fib(" + i + ") = " + fibonacci(i))
```

### GPU Parallel Processing
```vortex
// Parallel array processing
let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

@gpu fn process_array(arr: Array) -> Array:
    parallel i in 0..arr.length:
        arr[i] = arr[i] * arr[i]  // Square each element
    return arr

let result = process_array(data)
print("Squared array:")
print(result)
```

## 🐛 Troubleshooting

### Common Issues

1. **REPL not starting**
   ```bash
   # Try with explicit command
   cargo run --bin vortex-lang
   ```

2. **File not found errors**
   ```bash
   # Ensure file exists and has .vx extension
   ls -la *.vx
   cargo run example.vx
   ```

3. **GPU simulation not working**
   - GPU acceleration is simulated for development
   - Check console output for `[GPU]` messages
   - Use `:reset` in REPL to refresh GPU state

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Verbose compilation
cargo run --verbose
```

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch

# Auto-rebuild on changes
cargo watch -x build

# Run tests continuously
cargo watch -x test
```

## 📋 Roadmap

- [ ] **Real GPU Support**: CUDA/OpenCL integration
- [ ] **Package System**: Module imports and exports
- [ ] **Standard Library**: Built-in functions and utilities
- [ ] **Language Server**: IDE support with LSP
- [ ] **Debugger**: Step-through debugging
- [ ] **Performance Tools**: Profiling and optimization
- [ ] **WebAssembly**: Browser compilation target

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Rust community for excellent tooling
- GPU computing research community
- Contributors and early adopters

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/your-org/vortex-lang/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/vortex-lang/discussions)
- **Documentation**: [Project Wiki](https://github.com/your-org/vortex-lang/wiki)

---

**Made with ❤️ and ⚡ by the Gyanendra Thakur**
