# PyO3-Levenshtein

A high-performance Python library for calculating Levenshtein distance, implemented in Rust using [PyO3](https://github.com/PyO3/pyo3) and [Maturin](https://github.com/PyO3/maturin).

This project was originally created as an exercise of integrating Rust code into the Python ecosystem, providing significant performance improvements over pure Python implementations for computational tasks.

## Features

- **Fast**: Leverages Rust's performance for CPU-intensive string distance calculations.
- **Unicode Support**: Correctly handles Unicode characters (e.g., emojis, accented characters).
- **Type Safe**: Built with Rust's strong type system.

## Usage

Import the module and use the `levenshtein` function:

```python
import pyo3_levenshtein as lev

# Calculate distance between two strings
distance = lev.levenshtein("kitten", "sitting")
print(f"Distance: {distance}")  # Output: 3

# Unicode works correctly
print(lev.levenshtein("café", "cafe"))  # Output: 1
print(lev.levenshtein("🦀", "🐍"))      # Output: 1
```

## Benchmarks

Performance comparison of different Levenshtein distance implementations (run with `uv run pytest benchmarking/benchmarks.py --benchmark-max-time 10`):

| Implementation | Min (μs) | Max (μs) | Mean (μs) | Median (μs) | Std Dev | OPS (Kops/s) | Rounds | Relative Speed |
|----------------|----------|----------|-----------|-------------|---------|--------------|--------|----------------|
| C (python-Levenshtein) | 9.82 | 33.92 | 10.99 | 10.90 | 0.49 | 90.98 | 517,921 | 1.0x (baseline) |
| PyO3 (this project) | 23.55 | 96.64 | 26.05 | 26.01 | 1.50 | 38.38 | 183,362 | 2.4x slower |
| Pure Python | 1,143.89 | 2,525.57 | 1,271.82 | 1,269.32 | 52.94 | 0.79 | 7,729 | 115.7x slower |

**Key takeaways:**
- PyO3 implementation is **~48.8x faster** than pure Python implementation
- PyO3 implementation is **~2.4x slower** than the C-based `python-Levenshtein` package
- PyO3 provides a good balance between performance and maintainability with Rust's memory safety guarantees

## Development

### Prerequisites

- Rust (cargo)
- Python 3.10+
- [uv](https://github.com/astral-sh/uv) (recommended for project management)
- Maturin
  - Recommended installation route: `uv tool install maturin`

### Build and install

```bash
# Compile and install in editable mode
maturin develop --release
```

You can use `ipython` or write a simple script after installing to use the Rust-powered module:

```bash
uv run ipython
```

### Create a wheel for distribution

```bash
maturin build --release
```

### Testing

Rust unit tests:

```bash
cargo test
```
