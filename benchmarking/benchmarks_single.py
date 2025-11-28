import random
import string
from typing import Callable

import pytest
from Levenshtein import distance as c_levenshtein
from pyo3_levenshtein import levenshtein as pyo3_levenshten_

from .pure_python_impl import wf_levenshtein as py_levenshtein

TEST_DATASET_SIZE = 2**14
MAX_STRING_LENGTH = 50


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


def run_levenshtein(test_dataset: list[tuple[str, str, int]], distance_func: Callable[[str, str], int]) -> list[int]:
    """Run a levenshtein distance function on a test dataset.

    Args:
        test_dataset: List of tuples containing (str1, str2, expected_distance)
        distance_func: The levenshtein distance function to use

    Returns:
        List of computed distances
    """
    result: list[int] = []
    for str1, str2, _ in test_dataset:
        result.append(distance_func(str1, str2))
    return result


def test_py_levenshtein(test_dataset: list[tuple[str, str, int]]):
    res = run_levenshtein(test_dataset, py_levenshtein)
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_pyo3_levenshten(test_dataset: list[tuple[str, str, int]]):
    res = run_levenshtein(test_dataset, pyo3_levenshten_)
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_c_levenshten(test_dataset: list[tuple[str, str, int]]):
    res = run_levenshtein(test_dataset, c_levenshtein)
    assert len(res) == TEST_DATASET_SIZE
    for pred, (_, _, gt) in zip(res, test_dataset):
        assert pred == gt


def test_benchmark_py_levenshtein(benchmark, test_dataset: list[tuple[str, str, int]]):
    res = benchmark(lambda: run_levenshtein(test_dataset, py_levenshtein))
    assert len(res) == TEST_DATASET_SIZE


def test_benchmark_pyo3_levenshtein(benchmark, test_dataset: list[tuple[str, str, int]]):
    res = benchmark(lambda: run_levenshtein(test_dataset, pyo3_levenshten_))
    assert len(res) == TEST_DATASET_SIZE


def test_benchmark_c_levenshtein(benchmark, test_dataset: list[tuple[str, str, int]]):
    res = benchmark(lambda: run_levenshtein(test_dataset, c_levenshtein))
    assert len(res) == TEST_DATASET_SIZE
