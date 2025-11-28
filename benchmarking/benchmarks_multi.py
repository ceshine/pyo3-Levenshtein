from pyo3_levenshtein import levenshtein_batch as pyo3_levenshten_batch

from .benchmarks_single import test_dataset, TEST_DATASET_SIZE  # noqa


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
