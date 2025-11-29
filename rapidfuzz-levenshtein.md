# Levenshtein Distance Optimization Techniques in rapidfuzz-cpp

This document describes the optimization techniques used in the [rapidfuzz-cpp](https://github.com/rapidfuzz/rapidfuzz-cpp) library to speed up the calculation of Levenshtein distances.

## Table of Contents

1. [Bit-Parallel Algorithms](#1-bit-parallel-algorithms)
2. [SIMD Vectorization](#2-simd-vectorization)
3. [Early Termination & Pruning](#3-early-termination--pruning)
4. [mbleven Algorithm](#4-mbleven-algorithm-for-small-edit-distances)
5. [Memory Optimization](#5-memory-optimization)
6. [Algorithmic Optimizations](#6-algorithmic-optimizations)
7. [Pattern Match Vector Optimization](#7-pattern-match-vector-optimization)
8. [Low-Level Optimizations](#8-low-level-optimizations)
9. [Cache-Friendly Data Structures](#9-cache-friendly-data-structures)
10. [Performance Impact Summary](#performance-impact-summary)

---

## 1. Bit-Parallel Algorithms

### Hyyrö's Algorithm (2003)

**Primary Implementation**: `rapidfuzz/distance/Levenshtein_impl.hpp:259-322`

The bit-parallel algorithm uses 64-bit integers to process up to 64 characters in parallel, achieving significant speedup over traditional dynamic programming.

**Key Features**:
- **Time Complexity**: O(n) for strings ≤ 64 characters (compared to O(mn) for Wagner-Fischer)
- Encodes the dynamic programming matrix into bitvectors VP (vertical positive) and VN (vertical negative)
- Uses bitwise operations (AND, OR, XOR, shifts) instead of arithmetic operations

**Core Operations**:
```cpp
D0 = (((X & VP) + VP) ^ VP) | X | VN
HP = VN | ~(D0 | VP)
HN = D0 & VP
```

### Block-based Extension

**Function**: `levenshtein_hyrroe2003_block()` (`Levenshtein_impl.hpp:642-850`)

- Extends the bit-parallel algorithm to strings of any length
- Processes strings in 64-character chunks with carry propagation between blocks
- Maintains O(n) space complexity while handling arbitrary-length strings

---

## 2. SIMD Vectorization

### AVX2 Implementation

**Location**: `rapidfuzz/details/simd_avx2.hpp`

**Features**:
- Processes 4×64-bit values in parallel using 256-bit AVX2 registers
- Supports uint8_t, uint16_t, uint32_t, and uint64_t data types
- Custom popcount implementation using lookup tables (`Levenshtein_impl.hpp:345-358`)
- Vectorized bit operations for multiple strings simultaneously

### SSE2 Implementation

**Location**: `rapidfuzz/details/simd_sse2.hpp`

- Fallback for systems without AVX2 support
- Processes 2×64-bit values in parallel using 128-bit SSE2 registers
- Custom popcount using bit manipulation (lines 334-368)

**SIMD Function**: `levenshtein_hyrroe2003_simd()` (`Levenshtein_impl.hpp:325-417`)

---

## 3. Early Termination & Pruning

### Ukkonen's Band Optimization

**Location**: `Levenshtein_impl.hpp:681-817`

Ukkonen's algorithm optimizes computation by only calculating cells within a diagonal band, significantly reducing work when a maximum distance is known.

**How it works**:
- Only computes cells within a diagonal band of width 2×max+1
- Dynamically adjusts band boundaries based on the current score
- Early exits when the score exceeds the cutoff
- Tracks `first_block` and `last_block` to limit computation
- Uses conditions like: `if (scores[last_block] < max + word_size)` to determine band membership

**Performance**: Up to 50% reduction in computation for bounded searches

### Common Affix Removal

**Location**: `rapidfuzz/details/common_impl.hpp:49-81`

**Functions**:
- `remove_common_prefix()`: Strips matching prefixes
- `remove_common_suffix()`: Strips matching suffixes
- `remove_common_affix()`: Strips both

Common prefixes and suffixes don't affect the distance, so removing them reduces the problem size proportionally.

### Score Cutoff Optimization

Multiple early exit checks throughout the code:
- Length difference check (`Levenshtein_impl.hpp:148-149, 863, 922`)
- Minimum distance check based on length difference
- Band score checks (lines 463, 497, 574, 617)

---

## 4. mbleven Algorithm (for small edit distances)

**Location**: `rapidfuzz/distance/Levenshtein_impl.hpp:188-239`

**Function**: `levenshtein_mbleven2018()`

For cases where the maximum edit distance is very small (≤3), the mbleven algorithm provides orders of magnitude speedup.

**How it works**:
- Pre-computed lookup table for all possible edit sequences with distance ≤ 3
- Encoded as 8-bit integers with 2 bits per operation:
  - 01 = DELETE
  - 10 = INSERT
  - 11 = SUBSTITUTE
- Tries each possible edit sequence and finds the minimum
- Much faster than dynamic programming for small distances

**Usage**: Automatically selected when `score_cutoff < 4` (lines 904, 928)

**Lookup Table**:
```cpp
static constexpr std::array<std::array<uint8_t, 7>, 9> levenshtein_mbleven2018_matrix
```

---

## 5. Memory Optimization

### Pattern Match Vector (PMV)

**Location**: `rapidfuzz/details/PatternMatchVector.hpp`

Pre-computes and caches character positions as bitmasks for fast lookup during distance calculation.

**Features**:
- Optimized storage for ASCII characters (array of 256 uint64_t)
- HashMap for non-ASCII characters
- O(1) lookup time during distance calculation
- Stores character positions as a single 64-bit bitmask per character

### Single-row DP Storage

- **Traditional Wagner-Fischer**: Uses O(mn) space
- **This implementation**: Uses O(n) space by only keeping one row/column
- **Bit-parallel**: Uses O(n/64) space with bitvector representation

### Hirschberg's Algorithm

**Function**: `levenshtein_align_hirschberg()` (`Levenshtein_impl.hpp:1185-1213`)

Reduces memory from O(mn) to O(m+n) when computing edit operations for very large strings.

**Features**:
- Uses divide-and-conquer approach
- Only activated for large strings (matrix_size >= 1MB) (line 1198)
- Enables alignment computation for strings that would otherwise exceed available memory

---

## 6. Algorithmic Optimizations

### Adaptive Algorithm Selection

**Function**: `uniform_levenshtein_distance()` (`Levenshtein_impl.hpp:908-954`)

The library automatically selects the best algorithm based on input characteristics:

```
if (score_cutoff < 4)
    → Use mbleven2018
else if (s2.size() < 65)
    → Use bit-parallel Hyyrö
else if (full_band <= 64)
    → Use small-band optimization
else
    → Use block-based approach
```

This ensures optimal performance across different input scenarios without manual tuning.

### Score Hint Doubling

**Location**: Lines 881-895, 941-950

Progressive expansion strategy to avoid unnecessary computation:

1. Starts with a conservative estimate (score_hint)
2. Computes with limited band width
3. Doubles the hint if the actual score exceeds it
4. Repeats until accurate result is found

**Benefit**: Avoids unnecessary computation for cases where a good match exists

### Small Band Optimization

**Functions**: `levenshtein_hyrroe2003_small_band()` (`Levenshtein_impl.hpp:420-505, 507-636`)

Special case optimization when band width ≤ 64:
- Computes only the diagonal band using a single 64-bit word
- Dynamically updates the pattern match vector online (lines 551-602)
- Avoids overhead of block-based approach for small bands

---

## 7. Pattern Match Vector Optimization

### Hybrid Storage Strategy

**Location**: `rapidfuzz/details/PatternMatchVector.hpp`

- Fast array lookup for characters 0-255 (O(1) access)
- Hashmap with optimized collision resolution for other characters
- Custom hash function based on CPython/Ruby strategy (lines 40-53)

### Block Pattern Match Vector

- Extends PMV to multiple 64-bit blocks for long strings
- Lazy allocation of hashmap (only when needed for non-ASCII)
- Minimizes memory overhead while maintaining performance

---

## 8. Low-Level Optimizations

### Intrinsic Functions

**Location**: `rapidfuzz/details/intrinsics.hpp`

Custom implementations optimized for different platforms:

- `popcount()`: Count set bits (lines 72-99)
- `countr_zero()`: Count trailing zeros (lines 156-200)
- `blsi()`, `blsr()`, `blsmsk()`: Bit manipulation (lines 123-153)
- Safe shift operations avoiding undefined behavior (lines 41-53)

### Compiler-specific Optimizations

- Uses compiler intrinsics where available
- Platform-specific implementations (MSVC vs GCC/Clang)
- SIMD alignment directives (`alignas()`)
- Enables aggressive compiler optimizations

### Loop Unrolling

**Location**: `intrinsics.hpp:202-226`

- Template-based compile-time loop unrolling
- Used extensively in SIMD code and bit-parallel algorithms
- Reduces loop overhead and enables better instruction pipelining
- Allows compiler to perform additional optimizations

---

## 9. Cache-Friendly Data Structures

### Aligned Memory Allocation

**Location**: `rapidfuzz/details/common.hpp:71-94`

- Custom aligned allocator functions
- Ensures SIMD vectors are properly aligned:
  - 16-byte alignment for SSE2
  - 32-byte alignment for AVX2
- Prevents performance penalties from unaligned memory access

### Matrix Layout

**`ShiftedBitMatrix`**: Compact bit-level storage
- Row-major layout for better cache locality
- Dynamic offset support for Ukkonen banding
- Minimizes cache misses during DP computation

---

## Performance Impact Summary

The combination of these techniques provides substantial performance improvements over the traditional Wagner-Fischer algorithm:

### Speedup Factors

1. **Bit-parallel algorithms**: 8-64× speedup for short strings
2. **SIMD vectorization**: 2-4× additional speedup when comparing multiple strings
3. **Ukkonen banding**: Up to 50% reduction in computation for bounded searches
4. **mbleven**: Orders of magnitude faster for small edit distances (≤3)
5. **Common affix removal**: Proportional speedup based on affix length
6. **Adaptive selection**: Ensures optimal algorithm for each input
7. **Score cutoff/hint**: Avoids unnecessary computation in search scenarios

### Complexity Comparison

| Algorithm | Time Complexity | Space Complexity |
|-----------|----------------|------------------|
| Wagner-Fischer (baseline) | O(mn) | O(mn) |
| Optimized (string ≤ 64 chars) | O(n) | O(n) |
| Optimized (with cutoff k) | O(k·n) | O(n) |
| mbleven (distance ≤ 3) | O(n) | O(1) |
| Hirschberg (alignment) | O(mn) | O(m+n) |

### Real-world Impact

- **Short string comparisons**: 10-100× faster than naive implementation
- **Fuzzy search operations**: 5-50× faster with score cutoffs
- **Memory usage**: Constant factor reduction (often 100× less memory)
- **Scalability**: Can handle much larger strings within memory limits

---

## References

- Hyyrö, Heikki (2003): "A Bit-Vector Algorithm for Computing Levenshtein and Damerau Edit Distances"
- Ukkonen, Esko (1985): "Algorithms for Approximate String Matching"
- Hirschberg, Daniel S. (1975): "A linear space algorithm for computing maximal common subsequences"
- Myers, Gene (1999): "A fast bit-vector algorithm for approximate string matching based on dynamic programming"

---

## Implementation Files

Key files containing these optimizations:

- `rapidfuzz/distance/Levenshtein_impl.hpp`: Main algorithm implementations
- `rapidfuzz/details/PatternMatchVector.hpp`: Pattern matching optimization
- `rapidfuzz/details/simd_avx2.hpp`: AVX2 SIMD implementations
- `rapidfuzz/details/simd_sse2.hpp`: SSE2 SIMD implementations
- `rapidfuzz/details/intrinsics.hpp`: Low-level intrinsic functions
- `rapidfuzz/details/common_impl.hpp`: Common utility functions
- `rapidfuzz/details/common.hpp`: Memory allocation and data structures
