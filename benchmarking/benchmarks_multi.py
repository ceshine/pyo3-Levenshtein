from typing import Callable, cast

import joblib
from pyo3_levenshtein import levenshtein_batch as pyo3_levenshten_batch

from .benchmarks_single import test_dataset, TEST_DATASET_SIZE  # noqa
from .pure_python_impl import wf_levenshtein as py_levenshtein


def run_levenshtein_in_parallel(
    test_dataset: list[tuple[str, str, int]], distance_func: Callable[[str, str], int], num_workers: int = -1
) -> list[int]:
    """Run a levenshtein distance function on a test dataset.

    Args:
        test_dataset: List of tuples containing (str1, str2, expected_distance)
        distance_func: The levenshtein distance function to use
        num_workers: Number of workers to use for parallel processing

    Returns:
        List of computed distances
    """
    result: list[int] = cast(
        list[int],
        joblib.Parallel(n_jobs=num_workers, backend="loky")(
            joblib.delayed(distance_func)(str1, str2) for str1, str2, _ in test_dataset
        ),
    )
    return result


def test_py_benchmark_levenshtein_6p(benchmark, test_dataset: list[tuple[str, str, int]]):
    res = benchmark(lambda: run_levenshtein_in_parallel(test_dataset, py_levenshtein, num_workers=6))
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_py_benchmark_levenshtein_12p(benchmark, test_dataset: list[tuple[str, str, int]]):
    res = benchmark(lambda: run_levenshtein_in_parallel(test_dataset, py_levenshtein, num_workers=12))
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_py_benchmark_levenshtein_20p(benchmark, test_dataset: list[tuple[str, str, int]]):
    res = benchmark(lambda: run_levenshtein_in_parallel(test_dataset, py_levenshtein, num_workers=20))
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_py_levenshtein_8p(test_dataset: list[tuple[str, str, int]]):
    """Test for correctness"""
    res = run_levenshtein_in_parallel(test_dataset, py_levenshtein, num_workers=8)
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_pyo3_batch_levenshtein_8p(test_dataset: list[tuple[str, str, int]]):
    pairs = [(s1, s2) for s1, s2, _ in test_dataset]
    res = run_pyo3_batch_levenshtein(pairs)
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def run_pyo3_batch_levenshtein(pairs: list[tuple[str, str]], num_threads: int | None = None) -> list[int]:
    """Run batch levenshtein on entire dataset at once."""
    return pyo3_levenshten_batch(pairs, num_threads=num_threads)


def test_pyo3_batch_levenshtein(test_dataset: list[tuple[str, str, int]]):
    pairs = [(s1, s2) for s1, s2, _ in test_dataset]
    res = run_pyo3_batch_levenshtein(pairs)
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_benchmark_pyo3_batch_default(benchmark, test_dataset: list[tuple[str, str, int]]):
    """Benchmark batch processing with default thread count."""
    pairs = [(s1, s2) for s1, s2, _ in test_dataset]
    res = benchmark(lambda: run_pyo3_batch_levenshtein(pairs))
    assert len(res) == TEST_DATASET_SIZE


def test_benchmark_pyo3_batch_2t(benchmark, test_dataset: list[tuple[str, str, int]]):
    """Benchmark batch processing with 2 threads."""
    pairs = [(s1, s2) for s1, s2, _ in test_dataset]
    res = benchmark(lambda: run_pyo3_batch_levenshtein(pairs, num_threads=2))
    assert len(res) == TEST_DATASET_SIZE


def test_benchmark_pyo3_batch_6t(benchmark, test_dataset: list[tuple[str, str, int]]):
    """Benchmark batch processing with 6 threads."""
    pairs = [(s1, s2) for s1, s2, _ in test_dataset]
    res = benchmark(lambda: run_pyo3_batch_levenshtein(pairs, num_threads=6))
    assert len(res) == TEST_DATASET_SIZE


def test_benchmark_pyo3_batch_12t(benchmark, test_dataset: list[tuple[str, str, int]]):
    """Benchmark batch processing with 12 threads."""
    pairs = [(s1, s2) for s1, s2, _ in test_dataset]
    res = benchmark(lambda: run_pyo3_batch_levenshtein(pairs, num_threads=12))
    assert len(res) == TEST_DATASET_SIZE


def test_benchmark_pyo3_batch_20t(benchmark, test_dataset: list[tuple[str, str, int]]):
    """Benchmark batch processing with 20 threads."""
    pairs = [(s1, s2) for s1, s2, _ in test_dataset]
    res = benchmark(lambda: run_pyo3_batch_levenshtein(pairs, num_threads=20))
    assert len(res) == TEST_DATASET_SIZE
