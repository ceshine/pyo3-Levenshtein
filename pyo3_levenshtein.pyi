from typing import List, Tuple, Optional

def levenshtein(s1: str, s2: str) -> int:
    """Calculates the Levenshtein distance between two strings.

    The Levenshtein distance is the minimum number of single-character edits
    (insertions, deletions, or substitutions) required to transform one string
    into another.

    Args:
        s1 (str): The first string.
        s2 (str): The second string.

    Returns:
        int: The Levenshtein distance between the two strings.
    """
    ...

def levenshtein_batch(pairs: List[Tuple[str, str]], num_threads: Optional[int] = None) -> List[int]:
    """Calculates Levenshtein distances for multiple string pairs in parallel.

    This function processes a list of string pairs concurrently using multiple threads,
    releasing Python's GIL to enable true parallelism.

    Args:
        pairs (List[Tuple[str, str]]): A list of string pairs (tuples) to process.
        num_threads (Optional[int]): Optional number of threads to use. If None, uses all available CPU cores.

    Returns:
        List[int]: A list of Levenshtein distances in the same order as the input pairs.

    Raises:
        ValueError: If num_threads is 0.
    """
    ...
