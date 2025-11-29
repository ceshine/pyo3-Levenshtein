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

### Batch Processing (Multi-threaded)

For calculating distances on multiple string pairs, use the parallel batch function:

```python
import pyo3_levenshtein as lev

# Prepare multiple string pairs
pairs = [
    ("kitten", "sitting"),
    ("saturday", "sunday"),
    ("hello", "world"),
]

# Process all pairs in parallel (uses all CPU cores by default)
distances = lev.levenshtein_batch(pairs)
print(distances)  # Output: [3, 3, 4]

# Control thread count explicitly
distances = lev.levenshtein_batch(pairs, num_threads=4)
```

The `levenshtein_batch` function:

- Releases Python's GIL for true parallel processing
- Uses Rayon's work-stealing scheduler for optimal load balancing
- Defaults to using all available CPU cores
- Returns results in the same order as input pairs
- Best for processing 1,000,000+ pairs at once

## Benchmarks

Performance comparison of different Levenshtein distance implementations (run with `uv run pytest benchmarking/benchmarks_*.py --benchmark-max-time 10`):

### System Specs

- **CPU**: 6 performance cores (12 hyper-threads) + 8 efficiency cores = 20 total logical processors
- **Default thread count**: 20 (Rayon uses all available CPU cores by default)

### Single-threaded Processing

Processing 1 million string pairs sequentially:

| Implementation | Min (ms) | Max (ms) | Mean (ms) | Median (ms) | Std Dev (ms) | OPS | Rounds | Relative Speed |
|----------------|---------:|---------:|----------:|------------:|-------------:|----:|-------:|---------------:|
| C (python-Levenshtein) | 4.23 | 8.90 | 4.41 | 4.38 | 0.17 | 226.9 | 2,272 | 1.0x (baseline) |
| PyO3 (this project) | 38.19 | 48.68 | 39.32 | 39.06 | 1.09 | 25.4 | 255 | 8.9x slower |
| Pure Python | 2,464.47 | 2,498.91 | 2,482.71 | 2,481.24 | 13.05 | 0.4 | 5 | 563.5x slower |

### Batch/Parallel Processing

#### PyO3 Multi-threaded Processing

Processing 1 million string pairs with different thread configurations:

| Implementation | Threads | Min (ms) | Max (ms) | Mean (ms) | Median (ms) | Std Dev (ms) | OPS | Rounds | Speedup vs PyO3 Single (39.32ms) |
|----------------|--------:|---------:|---------:|----------:|------------:|-------------:|----:|-------:|---------------------------------:|
| PyO3 Batch | Default | 5.29 | 10.64 | 6.39 | 6.17 | 0.82 | 156.4 | 1,336 | 6.1x faster |
| PyO3 Batch | 20 | 5.15 | 13.13 | 7.17 | 6.87 | 1.48 | 139.5 | 1,614 | 5.5x faster |
| PyO3 Batch | 12 | 6.05 | 13.06 | 6.54 | 6.41 | 0.54 | 152.8 | 1,544 | 6.0x faster |
| PyO3 Batch | 6 | 8.43 | 16.05 | 9.47 | 9.08 | 1.11 | 105.6 | 772 | 4.2x faster |
| PyO3 Batch | 2 | 21.06 | 36.34 | 22.62 | 22.42 | 1.26 | 44.2 | 415 | 1.7x faster |

#### Pure Python Multi-process Processing (joblib)

Processing 1 million string pairs with different process counts:

| Implementation | Processes | Min (ms) | Max (ms) | Mean (ms) | Median (ms) | Std Dev (ms) | OPS | Rounds | Speedup vs Pure Python Single (2,482.71ms) |
|----------------|----------:|---------:|---------:|----------:|------------:|-------------:|----:|-------:|--------------------------------------------:|
| Pure Python | 20 | 296.45 | 334.94 | 316.37 | 317.75 | 8.45 | 3.2 | 30 | 7.8x faster |
| Pure Python | 12 | 300.45 | 344.19 | 314.79 | 311.89 | 12.75 | 3.2 | 26 | 7.9x faster |
| Pure Python | 6 | 464.58 | 493.96 | 477.67 | 477.53 | 7.34 | 2.1 | 18 | 5.2x faster |

### Performance Summary

Comparison of best results from each approach:

| Approach | Best Mean Time (ms) | vs C Baseline |
|----------|--------------------:|--------------:|
| C (single-threaded) | 4.41 | 1.0x (baseline) |
| PyO3 Batch (12 threads) | 6.54 | 1.48x slower |
| PyO3 Single-threaded | 39.32 | 8.9x slower |
| Pure Python Parallel (12 processes) |  314.79 | 71.4x slower |
| Pure Python Single-threaded | 2,482.71 | 563.0x slower |

**Key takeaways:**

- PyO3 implementation is **~63x faster** than pure Python implementation (single-threaded)
- PyO3 implementation is **~8.9x slower** than the C-based `python-Levenshtein` package (single-threaded)
- PyO3 batch processing with default thread count is **~6.1x faster** than single-threaded PyO3
- **PyO3 batch processing (6.54ms with 12 threads) nearly matches the highly-optimized C implementation (4.41ms), being only ~1.48x slower.** Note that this compares our multi-threaded implementation against a single-threaded C baseline. While the C library is algorithmically more efficient per core, PyO3 achieves competitive wall-clock times through out-of-the-box parallelism.
- Pure Python parallel implementation (using joblib) shows ~7.9x speedup with 12 **processes** (not using threads because of the GIL) compared to single-threaded Pure Python, but still significantly slower than single-threaded PyO3
- PyO3 provides excellent parallelization efficiency with the GIL released, achieving near-linear speedup with thread count (see Additional Notes for more details)
- PyO3 provides a good balance between performance and maintainability with Rust's memory safety guarantees

**Additional Notes:**

- **Speedup trend with increasing thread count**: PyO3 batch processing shows strong scaling up to 12 threads (2 threads: 1.7x → 6 threads: 4.2x → 12 threads: 6.0x speedup), but exhibits diminishing returns beyond that point. Using 12 threads (6.54ms) provides roughly the same performance as using 20 threads (6.39ms default, 7.17ms fixed), indicating that the efficiency cores are not particularly useful for this task. The optimal thread count appears to be around 12, matching the number of hyper-threads on the performance cores.
- The [python-Levenshtein](https://github.com/rapidfuzz/python-Levenshtein) package uses the C++ [rapidfuzz](https://github.com/rapidfuzz/rapidfuzz-cpp) library, which provides a highly optimized implementation of the Levenshtein distance metric. It is said to provide an 8× to 64× speedup for shorter strings (fewer than 64 characters), which is consistent with a roughly 9× speedup compared with the simpler Wagner–Fischer algorithm implemented by our PyO3-based package.
- I asked Claude Sonnet 4.5 to summarize the optimization techniques used to calculate the Levenshtein distance. The results can be found in [rapidfuzz-levenshtein.md](./rapidfuzz-levenshtein.md).

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
maturin develop --release --uv
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

Rust unit tests (requires a Python virtual environment):

```bash
./run-cargo-tests.sh
```

The test setup automatically detects and uses the Python interpreter from your `.venv` environment, making it portable across different machines and Python installations.

Python tests with pytest:

```bash
uv run pytest
```
