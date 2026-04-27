# Vortex Language REPL Usage Guide

## 🚀 Quick Start

### Start the REPL
```bash
cargo run
```

### Execute a file
```bash
cargo run example.vx
```

## 📋 REPL Commands

| Command | Description |
|---------|-------------|
| `:help` | Show help message |
| `:exit` | Exit the REPL |
| `:clear` | Clear the screen |
| `:history` | Show command history |
| `:load <file>` | Load and execute a Vortex file |
| `:env` | Show current environment variables |
| `:reset` | Reset the interpreter state |

## 🔧 Interactive Features

### ✅ Command History
- Use ↑/↓ arrow keys to navigate through previous commands
- Command history is automatically saved to `vortex_history.txt`

### ✅ Colored Output
- Commands are highlighted in cyan
- Errors appear in red
- Success messages in green
- Warnings in yellow

### ✅ Multi-line Input
Start multi-line blocks with these patterns:
- `if condition:`
- `for variable in range:`
- `fn function_name():`
- `@gpu fn function_name():`
- `parallel variable in range:`

### ✅ Error Recovery
- Syntax errors don't crash the REPL
- Use Ctrl+C to cancel multi-line input
- Invalid commands show helpful error messages

## 🎯 Example Usage

### Basic Variables
```vortex
vortex> let x = 42
vortex> let mut y = 3.14
vortex> print(x)
Output: 42
```

### Functions
```vortex
vortex> fn add(a: Int, b: Int) -> Int:
...     return a + b
... 
vortex> let result = add(5, 3)
vortex> print(result)
Output: 8
```

### GPU Functions
```vortex
vortex> @gpu fn gpu_multiply(a: Int, b: Int):
...     return a * b
...
vortex> let gpu_result = gpu_multiply(6, 7)
[GPU] Executing function: gpu_multiply
[GPU] Function returned: 42
vortex> print(gpu_result)
Output: 42
```

### Conditionals
```vortex
vortex> if x > 40:
...     print("x is large")
... then x == 42:
...     print("x is exactly 42")
... else:
...     print("x is small")
...
Output: x is exactly 42
```

### Loops
```vortex
vortex> for i in 0..3:
...     print("Iteration:")
...     print(i)
...
Output: Iteration:
Output: 0
Output: Iteration:
Output: 1
Output: Iteration:
Output: 2
```

### Parallel Processing
```vortex
vortex> parallel i in 0..4:
...     print("Parallel iteration:")
...     print(i)
...
[GPU] Simulating parallel loop over range 0..4
[GPU::Sim] Iteration 0
[GPU::Sim] Iteration 1
[GPU::Sim] Iteration 2
[GPU::Sim] Iteration 3
```

### File Loading
```vortex
vortex> :load example.vx
Info: Loading file: example.vx
=== Variable Declarations ===
x = 5
y = 3.14
Success: File executed successfully.
```

## 🎮 Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑` | Previous command |
| `↓` | Next command |
| `Ctrl+C` | Cancel current input / Exit |
| `Ctrl+D` | Exit REPL |
| `Tab` | (Future: Auto-completion) |

## 🐛 Error Handling

### Syntax Errors
```vortex
vortex> let x =
Warning: No valid statements found.

vortex> print(undefined_variable)
Runtime Error: Undefined variable 'undefined_variable'
```

### Multi-line Recovery
```vortex
vortex> if x > 10:
...     print("test")
... ^C
vortex> # Multi-line input cancelled, back to normal prompt
```

## 📁 File Operations

### Save Session
Command history is automatically saved to `vortex_history.txt`

### Load Scripts
```bash
# Create a script file
echo 'let x = 100\nprint("Hello Vortex!")' > hello.vx

# Load in REPL
vortex> :load hello.vx
```

## 🔍 Debugging

### Environment Inspection
```vortex
vortex> :env
Current Environment:
Info: Environment inspection not yet implemented
Note: Variables and functions are stored internally
```

### Reset State
```vortex
vortex> :reset
Interpreter state reset.
```

## 🚦 Exit Options

```vortex
vortex> :exit
Goodbye!

# Or use Ctrl+D
vortex> # Ctrl+D pressed
Goodbye!

# Or use Ctrl+C
vortex> # Ctrl+C pressed
^C
```

## 🎨 Tips & Tricks

1. **Use `:clear` frequently** to keep your workspace clean
2. **Test functions interactively** before putting them in files
3. **Use `:history`** to review and reuse previous commands
4. **Leverage multi-line input** for complex function definitions
5. **Use `:reset`** if the interpreter state gets corrupted
6. **Load files with `:load`** to test larger programs quickly

## 🔧 Advanced Usage

### Combining File and Interactive Mode
1. Start with a base file: `:load base.vx`
2. Experiment interactively with modifications
3. Test new functions before adding them to files
4. Use `:reset` and `:load` to refresh from the file

### GPU Development Workflow
1. Define GPU functions interactively
2. Test with small datasets
3. Compare with CPU versions
4. Monitor GPU simulation output
5. Export working functions to files