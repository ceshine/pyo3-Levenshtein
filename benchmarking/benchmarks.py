import random
import string

import pytest

from Levenshtein import distance as c_levenshtein
from pure_python_impl import wf_levenshtein as py_levenshtein
from pyo3_levenshtein import levenshtein as pyo3_levenshten_

TEST_DATASET_SIZE = 10
MAX_STRING_LENGTH = 20


def build_test_dataset(num_pairs: int) -> list[tuple[str, str, int]]:
    """Generate a test dataset of random string pairs.

    Args:
        num_pairs(int): Number of string pairs to generate

    Returns:
        A list of tuples, where each tuple contains two random strings
    """
    # Character set: alphanumeric characters
    chars = string.ascii_letters + string.digits

    dataset: list[tuple[str, str, int]] = []
    for _ in range(num_pairs):
        # Generate two random strings with lengths between 1 and 100
        len1 = random.randint(1, MAX_STRING_LENGTH)
        len2 = random.randint(1, MAX_STRING_LENGTH)

        str1 = "".join(random.choice(chars) for _ in range(len1))
        str2 = "".join(random.choice(chars) for _ in range(len2))

        dist = c_levenshtein(str1, str2)
        dataset.append((str1, str2, dist))

    return dataset


@pytest.fixture(scope="session")
def test_dataset() -> list[tuple[str, str, int]]:
    """Fixture to generate a test dataset of random string pairs."""
    return build_test_dataset(TEST_DATASET_SIZE)


def run_py_levenshtein(test_dataset: list[tuple[str, str, int]]) -> list[int]:
    result: list[int] = []
    for str1, str2, _ in test_dataset:
        result.append(py_levenshtein(str1, str2))
    return result


def run_pyo3_levenshten(test_dataset: list[tuple[str, str, int]]) -> list[int]:
    result: list[int] = []
    for str1, str2, _ in test_dataset:
        result.append(pyo3_levenshten_(str1, str2))
    return result


def run_c_levenshten(test_dataset: list[tuple[str, str, int]]) -> list[int]:
    result: list[int] = []
    for str1, str2, _ in test_dataset:
        result.append(c_levenshtein(str1, str2))
    return result


def test_py_levenshtein(test_dataset):
    res = run_py_levenshtein(test_dataset)
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_pyo3_levenshten(test_dataset):
    res = run_pyo3_levenshten(test_dataset)
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_c_levenshten(test_dataset):
    res = run_c_levenshten(test_dataset)
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_benchmark_py_levenshtein(benchmark, test_dataset):
    res = benchmark(lambda: run_py_levenshtein(test_dataset))
    assert len(res) == TEST_DATASET_SIZE


def test_benchmark_pyo3_levenshtein(benchmark, test_dataset):
    res = benchmark(lambda: run_pyo3_levenshten(test_dataset))
    assert len(res) == TEST_DATASET_SIZE


def test_benchmark_c_levenshtein(benchmark, test_dataset):
    res = benchmark(lambda: run_c_levenshten(test_dataset))
    assert len(res) == TEST_DATASET_SIZE
