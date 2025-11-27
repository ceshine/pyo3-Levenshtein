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
