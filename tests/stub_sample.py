import pyo3_levenshtein

# Correct usage
dist_correct = pyo3_levenshtein.levenshtein("kitten", "sitting")
batch_dist_correct = pyo3_levenshtein.levenshtein_batch([("hello", "world"), ("foo", "bar")])

# Intentionally incorrect usage (type checker should catch these)

# 1. Passing an integer instead of a string for s1
dist_error_s1_int = pyo3_levenshtein.levenshtein(123, "sitting")  # type: ignore[reportArgumentType]

# 2. Passing an integer instead of a string for s2
dist_error_s2_int = pyo3_levenshtein.levenshtein("kitten", 456)  # type: ignore[reportArgumentType]

# 3. Passing a list of strings instead of list of tuples for pairs
batch_error_list_str = pyo3_levenshtein.levenshtein_batch(["kitten", "sitting"])  # type: ignore[reportArgumentType]

# 4. Passing an argument to num_threads that is not an int or None
batch_error_num_threads_str = pyo3_levenshtein.levenshtein_batch([("a", "b")], num_threads="two")  # type: ignore[reportArgumentType]

# 5. Expecting a string return type instead of int for levenshtein
result_type_mismatch_str: str = pyo3_levenshtein.levenshtein("a", "b")  # type: ignore[reportAssignmentType]

# 6. Expecting a single int return type instead of List[int] for levenshtein_batch
result_type_mismatch_list_int: int = pyo3_levenshtein.levenshtein_batch([("a", "b")])  # type: ignore[reportAssignmentType]
