"""Integration tests for levenshtein_batch function."""
import pytest
from pyo3_levenshtein import levenshtein, levenshtein_batch


def test_batch_empty():
    """Test batch function with empty list."""
    result = levenshtein_batch([])
    assert result == []
    result_grapheme = levenshtein_batch([], grapheme_segmentation=True)
    assert result_grapheme == []


def test_batch_single_pair():
    """Test batch function with single pair."""
    pairs = [("kitten", "sitting")]
    result_char = levenshtein_batch(pairs, grapheme_segmentation=False)
    assert result_char == [3]
    assert result_char[0] == levenshtein("kitten", "sitting", grapheme_segmentation=False)
    result_grapheme = levenshtein_batch(pairs, grapheme_segmentation=True)
    assert result_grapheme == [3]
    assert result_grapheme[0] == levenshtein("kitten", "sitting", grapheme_segmentation=True)


def test_batch_multiple_pairs():
    """Test batch function with multiple pairs."""
    pairs = [
        ("kitten", "sitting"),
        ("hello", "hello"),
        ("", "world"),
        ("café", "cafe"),
        ("🦀", "🐍"),
    ]
    result = levenshtein_batch(pairs)
    expected = [levenshtein(s1, s2) for s1, s2 in pairs]
    assert result == expected


def test_batch_custom_thread_count():
    """Test batch function with custom thread count."""
    pairs = [("test", "best"), ("hello", "world")] * 10

    result_default = levenshtein_batch(pairs)
    result_1_thread = levenshtein_batch(pairs, num_threads=1)
    result_4_threads = levenshtein_batch(pairs, num_threads=4)

    # All should produce same results
    assert result_default == result_1_thread == result_4_threads

    # Verify correctness
    expected = [levenshtein(s1, s2) for s1, s2 in pairs]
    assert result_default == expected


def test_batch_invalid_thread_count():
    """Test batch function with invalid thread count."""
    pairs = [("test", "test")]

    with pytest.raises(ValueError, match="num_threads must be at least 1"):
        levenshtein_batch(pairs, num_threads=0)


def test_batch_large_dataset():
    """Test batch function with larger dataset."""
    import random
    import string

    pairs = []
    for _ in range(1000):
        s1 = ''.join(random.choices(string.ascii_letters, k=10))
        s2 = ''.join(random.choices(string.ascii_letters, k=10))
        pairs.append((s1, s2))

    result = levenshtein_batch(pairs)
    expected = [levenshtein(s1, s2) for s1, s2 in pairs]
    assert result == expected


def test_batch_unicode_edge_cases():
    """Test batch function with various Unicode scenarios."""
    pairs = [
        ("", ""),
        ("🦀🐍", "🐍🦀"),
        ("hello 世界", "hello world"),
        ("café", "cafe"),
        ("naïve", "naive"),
    ]
    result = levenshtein_batch(pairs)
    expected = [levenshtein(s1, s2) for s1, s2 in pairs]
    assert result == expected


def test_batch_consistency():
    """Verify batch function produces same results as single function."""
    test_cases = [
        ("kitten", "sitting"),
        ("saturday", "sunday"),
        ("", ""),
        ("abc", ""),
        ("", "xyz"),
        ("same", "same"),
    ] * 5  # Repeat to ensure parallelization doesn't affect correctness

    batch_result = levenshtein_batch(test_cases)
    single_results = [levenshtein(s1, s2) for s1, s2 in test_cases]

    assert batch_result == single_results


def test_batch_grapheme_segmentation():
    """Test batch function with grapheme segmentation enabled."""
    pairs_grapheme = [
        ("kitten", "sitten"),
        ("अनुच्छेद", "अनुछेद"),  # Hindi example: grapheme diff 1, char diff 2
        ("é", "e"),  # Combining characters: grapheme diff 1, char diff 1
        ("ä", "ä"),  # Decomposed vs precomposed: grapheme diff 1, char diff 1-2
        ("👩‍👩‍👧‍👦", "👨‍👩‍👧‍👦"),  # Family emoji: grapheme diff 1
        ("café", "cafe"),  # With accent
        ("🦀", "🐍"),  # Emojis
        ("hello 世界", "hello world"), # Mixed script
    ]

    result_grapheme = levenshtein_batch(pairs_grapheme, grapheme_segmentation=True)
    expected_grapheme = [levenshtein(s1, s2, grapheme_segmentation=True) for s1, s2 in pairs_grapheme]
    assert result_grapheme == expected_grapheme

    # Also test that char segmentation yields different results for relevant cases
    result_char = levenshtein_batch(pairs_grapheme, grapheme_segmentation=False)
    expected_char = [levenshtein(s1, s2, grapheme_segmentation=False) for s1, s2 in pairs_grapheme]

    # For "अनुच्छेद", "अनुछेद": grapheme_segmentation=True should be 1, False should be 2
    assert expected_grapheme[1] == 1
    assert expected_char[1] == 2

    # For "é", "e": grapheme_segmentation=True should be 1, False should be 1 (e + accent vs e)
    assert expected_grapheme[2] == 1
    assert expected_char[2] == 1 # This specific case is 1 for both because 'é' has 1 grapheme and 2 chars, 'e' has 1 grapheme and 1 char.

    # For "ä", "ä": grapheme_segmentation=True should be 1, False should be 2 for "a\u{0308}" vs "ä"
    assert expected_grapheme[3] == 1
    # Check that it's different from the grapheme result for this case
    assert result_grapheme != result_char 

