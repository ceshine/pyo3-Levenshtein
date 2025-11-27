use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;

/// Calculates the Levenshtein distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions, or substitutions) required to transform one string
/// into another. This implementation uses dynamic programming with a 2D matrix.
///
/// # Examples
///
/// ```
/// let distance = levenshtein("kitten", "sitting");
/// assert_eq!(distance, 3);
/// ```
#[pyfunction]
pub fn levenshtein(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    // Handle edge cases: if either string is empty,
    // the distance is the length of the other string
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    // Convert strings to char vectors to handle Unicode properly
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    // Create a matrix where matrix[i][j] represents the minimum edits
    // needed to transform the first i characters of s1 into the first j characters of s2
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first column: distance from empty string to s1[0..i]
    // requires i deletions
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    // Initialize first row: distance from empty string to s2[0..j]
    // requires j insertions
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill in the rest of the matrix using dynamic programming
    for i in 1..=len1 {
        for j in 1..=len2 {
            // If characters match, no edit is needed (cost = 0)
            // Otherwise, substitution is needed (cost = 1)
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };

            // Take the minimum of three possible operations:
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion: remove char from s1
                    matrix[i][j - 1] + 1, // insertion: add char to s1
                ),
                matrix[i - 1][j - 1] + cost, // substitution: replace char in s1
            );
        }
    }

    // The bottom-right cell contains the final distance
    matrix[len1][len2]
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
#[pyfunction(signature = (pairs, num_threads=None))]
pub fn levenshtein_batch(
    py: Python<'_>,
    pairs: Vec<(String, String)>,
    num_threads: Option<usize>,
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
    let result = py.detach(|| {
        if let Some(threads) = num_threads {
            // Build custom thread pool
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .map_err(|e| format!("Failed to create thread pool: {}", e))
                .and_then(|pool| {
                    Ok(pool.install(|| {
                        pairs
                            .par_iter()
                            .map(|(s1, s2)| levenshtein(s1, s2))
                            .collect()
                    }))
                })
        } else {
            // Use default rayon pool (num CPUs)
            Ok(pairs
                .par_iter()
                .map(|(s1, s2)| levenshtein(s1, s2))
                .collect())
        }
    });

    result.map_err(|e| PyValueError::new_err(e))
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
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn test_empty_strings() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("hello", ""), 5);
        assert_eq!(levenshtein("", "world"), 5);
    }

    #[test]
    fn test_single_char_difference() {
        assert_eq!(levenshtein("kitten", "sitten"), 1);
    }

    #[test]
    fn test_classic_example() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn test_unicode() {
        assert_eq!(levenshtein("café", "cafe"), 1);
        assert_eq!(levenshtein("🦀", "🐍"), 1);
    }
}

#[cfg(test)]
mod batch_tests {
    use super::*;

    #[test]
    fn test_batch_empty() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let pairs: Vec<(String, String)> = vec![];
            let result = levenshtein_batch(py, pairs, None).unwrap();
            assert_eq!(result, Vec::<usize>::new());
        });
    }

    #[test]
    fn test_batch_single_pair() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let pairs = vec![("kitten".to_string(), "sitting".to_string())];
            let result = levenshtein_batch(py, pairs, None).unwrap();
            assert_eq!(result, vec![3]);
        });
    }

    #[test]
    fn test_batch_multiple_pairs() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let pairs = vec![
                ("kitten".to_string(), "sitting".to_string()),
                ("hello".to_string(), "hello".to_string()),
                ("".to_string(), "world".to_string()),
                ("café".to_string(), "cafe".to_string()),
            ];
            let result = levenshtein_batch(py, pairs, None).unwrap();
            assert_eq!(result, vec![3, 0, 5, 1]);
        });
    }

    #[test]
    fn test_batch_custom_threads() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let pairs = vec![
                ("kitten".to_string(), "sitting".to_string()),
                ("hello".to_string(), "world".to_string()),
            ];
            let result = levenshtein_batch(py, pairs, Some(2)).unwrap();
            assert_eq!(result.len(), 2);
            // Verify correctness
            assert_eq!(result[0], levenshtein("kitten", "sitting"));
            assert_eq!(result[1], levenshtein("hello", "world"));
        });
    }

    #[test]
    fn test_batch_invalid_thread_count() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let pairs = vec![("test".to_string(), "test".to_string())];
            let result = levenshtein_batch(py, pairs, Some(0));
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("num_threads must be at least 1")
            );
        });
    }

    #[test]
    fn test_batch_unicode() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let pairs = vec![
                ("🦀".to_string(), "🐍".to_string()),
                ("café".to_string(), "cafe".to_string()),
            ];
            let result = levenshtein_batch(py, pairs, None).unwrap();
            assert_eq!(result, vec![1, 1]);
        });
    }

    #[test]
    fn test_batch_consistency_with_single() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let test_cases = vec![
                ("kitten".to_string(), "sitting".to_string()),
                ("saturday".to_string(), "sunday".to_string()),
                ("".to_string(), "".to_string()),
                ("abc".to_string(), "".to_string()),
            ];

            let batch_results = levenshtein_batch(py, test_cases.clone(), None).unwrap();

            // Verify each result matches single function call
            for (i, (s1, s2)) in test_cases.iter().enumerate() {
                assert_eq!(batch_results[i], levenshtein(s1, s2));
            }
        });
    }
}
