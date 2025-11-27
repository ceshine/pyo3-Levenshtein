use pyo3::prelude::*;

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

/// PyO3 module definition for exposing functions to Python
#[pymodule]
fn pyo3_levenshtein(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(levenshtein, m)?)?;
    Ok(())
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
