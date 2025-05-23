# Vortex Language

Vortex is an interpreted programming language with built-in GPU acceleration capabilities, designed to provide easy access to parallel computation.

## Current Status

The current implementation uses a CPU-only fallback for system stability. CUDA-based GPU acceleration is under development.

## Running a Simple Test

To execute a simple test program:

```bash
# Build the project
cargo build

# Run the interpreter with a simple test program
cargo run
```

This will run the built-in test program that demonstrates basic functionality using the CPU implementation.

## Custom Test Programs

To run your own Vortex programs:

1. Edit the `src/main.rs` file to change the `source` string
2. Run with `cargo run`

## GPU Acceleration

GPU acceleration via CUDA is currently disabled to ensure system stability. Future releases will include proper GPU support.

## Syntax Example

```
// Define a GPU-accelerated function
@gpu fn add_vectors(a, b, n):
    return

// Call the function with arrays
let result = add_vectors(array_a, array_b, 4)
print(result)
```

## Development Notes

- If you experience system crashes, ensure that you're using the CPU-only implementation
- Memory usage is carefully managed to prevent leaks
- Full CUDA support is coming in a future update