"""Integration tests for levenshtein_batch function."""
import pytest
from pyo3_levenshtein import levenshtein, levenshtein_batch


def test_batch_empty():
    """Test batch function with empty list."""
    result = levenshtein_batch([])
    assert result == []


def test_batch_single_pair():
    """Test batch function with single pair."""
    pairs = [("kitten", "sitting")]
    result = levenshtein_batch(pairs)
    assert result == [3]
    assert result[0] == levenshtein("kitten", "sitting")


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
