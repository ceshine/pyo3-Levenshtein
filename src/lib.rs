use dashmap::DashMap;
use once_cell::sync::Lazy;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;
use smallvec::{SmallVec, smallvec};
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

/// Stack-allocated vector for small sizes (up to 32 elements), falling back to heap.
/// This significantly reduces allocation overhead for short strings.
type FastVec<T> = SmallVec<[T; 32]>;

/// Global cache of thread pools, keyed by thread count.
/// This allows reusing thread pools across multiple function calls,
/// avoiding the overhead of creating new thread pools each time.
static THREAD_POOL_CACHE: Lazy<DashMap<usize, Arc<rayon::ThreadPool>>> = Lazy::new(DashMap::new);

/// Gets an existing thread pool from the cache or creates a new one.
///
/// # Arguments
///
/// * `num_threads` - The number of threads for the pool
///
/// # Returns
///
/// An `Arc` to the thread pool, either from cache or newly created
///
/// # Errors
///
/// Returns an error string if thread pool creation fails
fn get_or_create_pool(num_threads: usize) -> Result<Arc<rayon::ThreadPool>, String> {
    // Try to get from cache first
    if let Some(pool) = THREAD_POOL_CACHE.get(&num_threads) {
        return Ok(Arc::clone(pool.value()));
    }

    // Create new pool if not in cache
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| format!("Failed to create thread pool: {}", e))?;

    let pool_arc = Arc::new(pool);

    // Insert into cache and return
    THREAD_POOL_CACHE.insert(num_threads, Arc::clone(&pool_arc));
    Ok(pool_arc)
}

/// Generic Levenshtein implementation for any type that implements PartialEq.
fn levenshtein_impl<T: PartialEq>(s1: &[T], s2: &[T]) -> usize {
    let (us1, us2) = if s1.len() < s2.len() {
        (s2, s1)
    } else {
        (s1, s2)
    };

    let rows = us1.len() + 1;
    let cols = us2.len() + 1;

    if rows == 1 {
        return cols - 1;
    } else if cols == 1 {
        return rows - 1;
    }

    let mut prev: FastVec<usize> = (0..cols).collect();
    let mut cur: FastVec<usize> = smallvec![0; cols];

    for r in 1..rows {
        cur[0] = r;
        for c in 1..cols {
            let del_or_ins = std::cmp::min(prev[c] + 1, cur[c - 1] + 1);
            let edit = prev[c - 1] + (if us1[r - 1] == us2[c - 1] { 0 } else { 1 });
            cur[c] = std::cmp::min(del_or_ins, edit);
        }
        std::mem::swap(&mut prev, &mut cur);
    }

    prev[cols - 1]
}

/// Calculates the Levenshtein distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions, or substitutions) required to transform one string
/// into another. This implementation uses dynamic programming with a 2D matrix.
///
/// # Arguments
///
/// * `s1` - The first string.
/// * `s2` - The second string.
/// * `grapheme_segmentation` - If true, strings are segmented into Unicode Grapheme Clusters before
///   calculating the distance. This is more accurate for languages with complex scripts but comes
///   with some performance penalties. Defaults to `false`.
///
/// # Examples
///
/// ```
/// use pyo3_levenshtein::levenshtein;
/// let distance = levenshtein("kitten", "sitting", false);
/// assert_eq!(distance, 3);
/// ```
///
/// ```
/// use pyo3_levenshtein::levenshtein;
/// // "अनुच्छेद" (article) in Hindi has 4 grapheme clusters
/// // "अनुच्छेद" (article) in Hindi has 7 characters
/// // "अनुछेद" (article, misspelled) in Hindi has 6 characters
/// let distance = levenshtein("अनुच्छेद", "अनुछेद", true); // Grapheme segmentation
/// assert_eq!(distance, 1);
/// let distance_char = levenshtein("अनुच्छेद", "अनुछेद", false); // Character segmentation
/// assert_eq!(distance_char, 2); // In this specific example, the result is 2 for char segmentation
/// ```
#[pyfunction(signature = (s1, s2, grapheme_segmentation = false))]
pub fn levenshtein(s1: &str, s2: &str, grapheme_segmentation: bool) -> usize {
    if s1 == s2 {
        return 0;
    }

    if grapheme_segmentation {
        let us1: FastVec<String> = UnicodeSegmentation::graphemes(s1, true)
            .map(|g| g.to_string())
            .collect();
        let us2: FastVec<String> = UnicodeSegmentation::graphemes(s2, true)
            .map(|g| g.to_string())
            .collect();
        levenshtein_impl(&us1, &us2)
    } else {
        let us1: FastVec<char> = s1.chars().collect();
        let us2: FastVec<char> = s2.chars().collect();
        levenshtein_impl(&us1, &us2)
    }
}

/// Calculates Levenshtein distances for multiple string pairs in parallel.
///
/// This function processes a list of string pairs concurrently using multiple threads,
/// releasing Python's GIL to enable true parallelism. It uses Rayon's work-stealing
/// scheduler for optimal load balancing across CPU cores.
///
/// # Arguments
///
/// * `pairs` - A vector of string pairs (tuples) to process
/// * `num_threads` - Optional number of threads to use. If None, uses all available CPU cores
/// * `grapheme_segmentation` - If true, strings are segmented into Unicode Grapheme Clusters before
///   calculating the distance. This is more accurate for languages with complex scripts but comes
///   with some performance penalties. Defaults to `false`.
///
/// # Returns
///
/// A vector of Levenshtein distances in the same order as the input pairs
///
/// # Errors
///
/// Returns `PyValueError` if:
/// * `num_threads` is 0
/// * Thread pool creation fails
///
/// # Examples
///
/// ```python
/// import pyo3_levenshtein as lev
///
/// pairs = [("kitten", "sitting"), ("hello", "world")]
/// distances = lev.levenshtein_batch(pairs)  # Uses all CPU cores
/// distances = lev.levenshtein_batch(pairs, num_threads=4)  # Uses 4 threads
/// ```
/// ```python
/// import pyo3_levenshtein as lev
///
/// pairs = [("अनुच्छेद", "अनुछेद")]
/// distances = lev.levenshtein_batch(pairs, grapheme_segmentation=True)
/// ```
#[pyfunction(signature = (pairs, num_threads=None, grapheme_segmentation = false))]
pub fn levenshtein_batch(
    py: Python<'_>,
    pairs: Vec<(String, String)>,
    num_threads: Option<usize>,
    grapheme_segmentation: bool,
) -> PyResult<Vec<usize>> {
    // Handle empty input
    if pairs.is_empty() {
        return Ok(Vec::new());
    }

    // Validate thread count if specified
    if let Some(threads) = num_threads {
        if threads == 0 {
            return Err(PyValueError::new_err("num_threads must be at least 1"));
        }
    }

    // Release GIL and perform computation
    // The computation is entirely in Rust and doesn't need Python access
    py.detach(|| {
        if let Some(threads) = num_threads {
            // Use cached thread pool for the specified thread count
            let pool = get_or_create_pool(threads).map_err(PyValueError::new_err)?;

            Ok(pool.install(|| {
                pairs
                    .par_iter()
                    .map(|(s1, s2)| levenshtein(s1, s2, grapheme_segmentation))
                    .collect()
            }))
        } else {
            // Use the global thread pool (lazily initialized and reused by Rayon)
            // This avoids creating a new thread pool on every call
            Ok(pairs
                .par_iter()
                .map(|(s1, s2)| levenshtein(s1, s2, grapheme_segmentation))
                .collect())
        }
    })
}

#[pymodule]
mod pyo3_levenshtein {
    #[pymodule_export]
    use super::levenshtein;

    #[pymodule_export]
    use super::levenshtein_batch;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_strings() {
        assert_eq!(levenshtein("hello", "hello", false), 0);
        assert_eq!(levenshtein("hello", "hello", true), 0);
    }

    #[test]
    fn test_empty_strings() {
        assert_eq!(levenshtein("", "", false), 0);
        assert_eq!(levenshtein("hello", "", false), 5);
        assert_eq!(levenshtein("", "world", false), 5);
        assert_eq!(levenshtein("", "", true), 0);
        assert_eq!(levenshtein("hello", "", true), 5);
        assert_eq!(levenshtein("", "world", true), 5);
    }

    #[test]
    fn test_single_char_difference() {
        assert_eq!(levenshtein("kitten", "sitten", false), 1);
        assert_eq!(levenshtein("kitten", "sitten", true), 1);
    }

    #[test]
    fn test_classic_example() {
        assert_eq!(levenshtein("kitten", "sitting", false), 3);
        assert_eq!(levenshtein("kitten", "sitting", true), 3);
    }

    #[test]
    fn test_unicode_char_segmentation() {
        assert_eq!(levenshtein("café", "cafe", false), 1);
        assert_eq!(levenshtein("🦀", "🐍", false), 1);
        // Test cases where character count != grapheme count
        // "अनुच्छेद" (article) in Hindi has 7 characters
        // "अनुछेद" (article, misspelled) in Hindi has 6 characters
        // Distance by character is 2 (as confirmed by Python)
        assert_eq!(levenshtein("अनुच्छेद", "अनुछेद", false), 2);
        // "niño" (child) has 4 characters
        // "nino" has 4 characters
        // The difference is 1 character ('ñ' vs 'n')
        assert_eq!(levenshtein("niño", "nino", false), 1);
        // Combining characters: "é" is 'e' + combining acute accent (U+0301)
        // Character segmentation: "e\u{0301}" (2 chars) vs "e" (1 char) -> distance 1 (as confirmed by Python)
        assert_eq!(levenshtein("e\u{0301}", "e", false), 1);
        // 'ä' is U+00E4 (1 char), 'a\u{0308}' is U+0061 U+0308 (2 chars)
        assert_eq!(levenshtein("ä", "a\u{0308}", false), 2);
    }

    #[test]
    fn test_unicode_grapheme_segmentation() {
        // "café" has 4 graphemes, "cafe" has 4 graphemes. Distance 1.
        assert_eq!(levenshtein("café", "cafe", true), 1);
        // "🦀" has 1 grapheme, "🐍" has 1 grapheme. Distance 1.
        assert_eq!(levenshtein("🦀", "🐍", true), 1);
        // "अनुच्छेद" (article) in Hindi has 4 grapheme clusters
        // "अनुछेद" (article, misspelled) in Hindi has 3 grapheme clusters
        // Distance by grapheme is 1
        assert_eq!(levenshtein("अनुच्छेद", "अनुछेद", true), 1);
        // "niño" has 4 graphemes, "nino" has 4 graphemes. Distance 1.
        assert_eq!(levenshtein("niño", "nino", true), 1);
        // Combining characters: "é" (1 grapheme) vs "e" (1 grapheme) -> distance 1
        assert_eq!(levenshtein("e\u{0301}", "e", true), 1);
        // 'ä' is U+00E4 (1 grapheme), 'a\u{0308}' is U+0061 U+0308 (1 grapheme). Distance 1.
        // Even though they are canonically equivalent, `unicode-segmentation` considers them
        // distinct grapheme clusters because their byte representation is different.
        // A true canonical equivalence check would require normalization, which is beyond
        // simple grapheme segmentation.
        assert_eq!(levenshtein("ä", "a\u{0308}", true), 1);

        // Test with a more complex grapheme cluster example
        // "👩‍👩‍👧‍👦" (family: woman, woman, girl, boy) is 1 grapheme cluster (using ZWJ)
        let s1 = "👩‍👩‍👧‍👦"; // Family emoji (1 grapheme cluster)
        let s2 = "👨‍👩‍👧‍👦"; // Family emoji (different head, 1 grapheme cluster)
        assert_eq!(levenshtein(s1, s2, true), 1); // Expected 1 edit to change a component
    }
}

#[cfg(test)]
mod batch_tests {
    use super::*;

    #[test]
    fn test_batch_empty() {
        Python::initialize();
        Python::attach(|py| {
            let pairs: Vec<(String, String)> = vec![];
            let result = levenshtein_batch(py, pairs.clone(), None, false).unwrap();
            assert_eq!(result, Vec::<usize>::new());
            let result_grapheme = levenshtein_batch(py, pairs, None, true).unwrap();
            assert_eq!(result_grapheme, Vec::<usize>::new());
        });
    }

    #[test]
    fn test_batch_single_pair() {
        Python::initialize();
        Python::attach(|py| {
            let pairs = vec![("kitten".to_string(), "sitting".to_string())];
            let result_char = levenshtein_batch(py, pairs.clone(), None, false).unwrap();
            assert_eq!(result_char, vec![3]);
            let result_grapheme = levenshtein_batch(py, pairs, None, true).unwrap();
            assert_eq!(result_grapheme, vec![3]);
        });
    }

    #[test]
    fn test_batch_multiple_pairs() {
        Python::initialize();
        Python::attach(|py| {
            let pairs_char = vec![
                ("kitten".to_string(), "sitting".to_string()),
                ("hello".to_string(), "hello".to_string()),
                ("".to_string(), "world".to_string()),
                ("café".to_string(), "cafe".to_string()),
            ];
            let result_char = levenshtein_batch(py, pairs_char, None, false).unwrap();
            assert_eq!(result_char, vec![3, 0, 5, 1]);

            let pairs_grapheme = vec![
                ("kitten".to_string(), "sitten".to_string()),
                ("अनुच्छेद".to_string(), "अनुछेद".to_string()), // Grapheme diff 1
                ("e\u{0301}".to_string(), "e".to_string()), // Grapheme diff 1
                ("ä".to_string(), "a\u{0308}".to_string()), // Grapheme diff 1
                ("👩‍👩‍👧‍👦".to_string(), "👨‍👩‍👧‍👦".to_string()),      // Grapheme diff 1
            ];
            let result_grapheme = levenshtein_batch(py, pairs_grapheme, None, true).unwrap();
            assert_eq!(result_grapheme, vec![1, 1, 1, 1, 1]);
        });
    }

    #[test]
    fn test_batch_custom_threads() {
        Python::initialize();
        Python::attach(|py| {
            let pairs = vec![
                ("kitten".to_string(), "sitting".to_string()),
                ("hello".to_string(), "world".to_string()),
            ];
            let result_char = levenshtein_batch(py, pairs.clone(), Some(2), false).unwrap();
            assert_eq!(result_char.len(), 2);
            assert_eq!(result_char[0], levenshtein("kitten", "sitting", false));
            assert_eq!(result_char[1], levenshtein("hello", "world", false));

            let result_grapheme = levenshtein_batch(py, pairs, Some(2), true).unwrap();
            assert_eq!(result_grapheme.len(), 2);
            assert_eq!(result_grapheme[0], levenshtein("kitten", "sitting", true));
            assert_eq!(result_grapheme[1], levenshtein("hello", "world", true));
        });
    }

    #[test]
    fn test_batch_invalid_thread_count() {
        Python::initialize();
        Python::attach(|py| {
            let pairs = vec![("test".to_string(), "test".to_string())];
            let result = levenshtein_batch(py, pairs.clone(), Some(0), false);
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("num_threads must be at least 1")
            );

            let result_grapheme = levenshtein_batch(py, pairs, Some(0), true);
            assert!(result_grapheme.is_err());
            assert!(
                result_grapheme
                    .unwrap_err()
                    .to_string()
                    .contains("num_threads must be at least 1")
            );
        });
    }

    #[test]
    fn test_batch_unicode_grapheme() {
        Python::initialize();
        Python::attach(|py| {
            let pairs = vec![
                ("🦀".to_string(), "🐍".to_string()),
                ("café".to_string(), "cafe".to_string()),
                ("अनुच्छेद".to_string(), "अनुछेद".to_string()),
                ("e\u{0301}".to_string(), "e".to_string()),
            ];
            let result = levenshtein_batch(py, pairs, None, true).unwrap();
            assert_eq!(result, vec![1, 1, 1, 1]);
        });
    }

    #[test]
    fn test_batch_consistency_with_single() {
        Python::initialize();
        Python::attach(|py| {
            let test_cases = vec![
                ("kitten".to_string(), "sitting".to_string()),
                ("saturday".to_string(), "sunday".to_string()),
                ("".to_string(), "".to_string()),
                ("abc".to_string(), "".to_string()),
                ("अनुच्छेद".to_string(), "अनुछेद".to_string()),
                ("e\u{0301}".to_string(), "e".to_string()),
                ("👩‍👩‍👧‍👦".to_string(), "👨‍👩‍👧‍👦".to_string()),
            ];

            // Test with char segmentation
            let batch_results_char =
                levenshtein_batch(py, test_cases.clone(), None, false).unwrap();
            for (i, (s1, s2)) in test_cases.iter().enumerate() {
                assert_eq!(batch_results_char[i], levenshtein(s1, s2, false));
            }

            // Test with grapheme segmentation
            let batch_results_grapheme =
                levenshtein_batch(py, test_cases.clone(), None, true).unwrap();
            for (i, (s1, s2)) in test_cases.iter().enumerate() {
                assert_eq!(batch_results_grapheme[i], levenshtein(s1, s2, true));
            }
        });
    }
}
