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

# For complex scripts or extended emoji sequences, use grapheme segmentation
# "अनुच्छेद" (article) has 4 grapheme clusters but 7 characters
# "अनुछेद" (misspelled) has 3 grapheme clusters but 6 characters
# Distance: 1 grapheme edit vs 2 character edits
print(lev.levenshtein("अनुच्छेद", "अनुछेद", grapheme_segmentation=True))  # Output: 1
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

## Changelog

### 0.3.0 (Unreleased)

- **Features**:
  - Added support for Unicode grapheme cluster segmentation. You can now pass `grapheme_segmentation=True` to `levenshtein` and `levenshtein_batch` to correctly handle complex scripts (like Devanagari) and extended emoji sequences.
  - Optimized memory allocation for short strings using `SmallVec`, reducing overhead for common use cases.
  - **Improved space complexity**: The Levenshtein calculation now uses `O(min(M, N))` space, a significant improvement over the previous `O(M * N)` complexity, where M and N are the lengths of the input strings.
- **Performance**:
  - Significant speedup in single-threaded `levenshtein` (~1.7x faster, 39.32ms -> 23.62ms).
  - Improved batch processing performance across all thread counts.
  - 12-thread batch processing (4.64ms) now effectively matches the single-threaded C implementation (4.57ms).
  - **Note**: These benchmarks are run with `grapheme_segmentation=False` (default). Enabling grapheme segmentation incurs performance overhead, bringing speeds closer to version 0.2.x levels.
- **Internal**:
  - Refactored core logic into a generic `levenshtein_impl` to support both character-based and grapheme-based calculations efficiently.
  - The 0.3.0 optimizations were substantially inspired by the implementation found in the [`jellyfish` package](https://github.com/jamesturk/jellyfish/blob/b21046a4f0c490f2c20548ba9b2c6c15fe120847/src/levenshtein.rs).
  - Added `unicode-segmentation` and `smallvec` dependencies.

## Benchmarks (Version 0.3.0)

Performance comparison of different Levenshtein distance implementations (run with `uv run pytest benchmarking/benchmarks_*.py --benchmark-max-time 10`):

### System Specs

- **CPU**: 6 performance cores (12 hyper-threads) + 8 efficiency cores = 20 total logical processors
- **Default thread count**: 20 (Rayon uses all available CPU cores by default)

### Single-threaded Processing

Processing 1 million string pairs sequentially:

| Implementation | Min (ms) | Max (ms) | Mean (ms) | Median (ms) | Std Dev (ms) | OPS | Rounds | Relative Speed |
|----------------|---------:|---------:|----------:|------------:|-------------:|----:|-------:|---------------:|
| C (python-Levenshtein) | 4.32 | 4.90 | 4.57 | 4.57 | 0.11 | 218.9 | 2,170 | 1.0x (baseline) |
| PyO3 (this project) | 22.43 | 32.60 | 23.62 | 23.52 | 0.86 | 42.3 | 425 | 5.2x slower |
| Pure Python | 2,500.95 | 2,621.48 | 2,534.94 | 2,515.97 | 49.12 | 0.4 | 5 | 554.7x slower |

### Batch/Parallel Processing

#### PyO3 Multi-threaded Processing

Processing 1 million string pairs with different thread configurations:

| Implementation | Threads | Min (ms) | Max (ms) | Mean (ms) | Median (ms) | Std Dev (ms) | OPS | Rounds | Speedup vs PyO3 Single (23.62ms) |
|----------------|--------:|---------:|---------:|----------:|------------:|-------------:|----:|-------:|---------------------------------:|
| PyO3 Batch | Default | 3.64 | 12.35 | 6.01 | 5.92 | 1.41 | 166.3 | 2,090 | 3.9x faster |
| PyO3 Batch | 20 | 3.64 | 11.70 | 5.74 | 5.60 | 1.43 | 174.3 | 2,263 | 4.1x faster |
| PyO3 Batch | 12 | 4.09 | 13.08 | 4.64 | 4.41 | 0.76 | 215.5 | 1,988 | 5.1x faster |
| PyO3 Batch | 6 | 5.57 | 13.13 | 6.36 | 6.11 | 0.83 | 157.2 | 1,394 | 3.7x faster |
| PyO3 Batch | 2 | 12.81 | 23.96 | 13.46 | 13.31 | 0.74 | 74.3 | 700 | 1.8x faster |

#### Pure Python Multi-process Processing (joblib)

Processing 1 million string pairs with different process counts:

| Implementation | Processes | Min (ms) | Max (ms) | Mean (ms) | Median (ms) | Std Dev (ms) | OPS | Rounds | Speedup vs Pure Python Single (2,534.94ms) |
|----------------|----------:|---------:|---------:|----------:|------------:|-------------:|----:|-------:|--------------------------------------------:|
| Pure Python | 20 | 290.86 | 337.54 | 308.97 | 305.26 | 12.06 | 3.2 | 30 | 8.2x faster |
| Pure Python | 12 | 297.32 | 333.31 | 306.20 | 303.37 | 8.92 | 3.3 | 25 | 8.3x faster |
| Pure Python | 6 | 459.27 | 485.47 | 469.10 | 466.30 | 6.83 | 2.1 | 20 | 5.4x faster |

### Performance Summary

Comparison of best results from each approach:

| Approach | Best Mean Time (ms) | vs C Baseline |
|----------|--------------------:|--------------:|
| C (single-threaded) | 4.57 | 1.0x (baseline) |
| PyO3 Batch (12 threads) | 4.64 | 1.02x slower (virtually identical) |
| PyO3 Single-threaded | 23.62 | 5.2x slower |
| Pure Python Parallel (12 processes) | 306.20 | 67.0x slower |
| Pure Python Single-threaded | 2,534.94 | 554.7x slower |

**Key takeaways:**

- PyO3 implementation is **~107x faster** than pure Python implementation (single-threaded)
- PyO3 implementation is **~5.2x slower** than the C-based `python-Levenshtein` package (single-threaded)
- PyO3 batch processing with 12 threads is **~5.1x faster** than single-threaded PyO3
- **PyO3 batch processing (4.64ms with 12 threads) effectively matches the highly-optimized C implementation (4.57ms).**
- PyO3 provides a good balance between performance and maintainability with Rust's memory safety guarantees

**Additional Notes:**

- **Speedup trend with increasing thread count**: PyO3 batch processing shows strong scaling. 12 threads (4.64ms) appears to be the optimal configuration, matching the number of hyper-threads on the performance cores.

## Benchmarks (Legacy; Version 0.2.x)

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
