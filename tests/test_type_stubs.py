import subprocess
import os


def test_type_stub_correctness():
    """Verifies type stub correctness using basedpyright."""

    # Run basedpyright on the temporary test file
    # We need to run it from the project root to ensure pyright.toml is picked up
    # And the stubPath configuration.
    project_root = os.getcwd()  # This should be /home/ceshine/codeમ્પ/personal-projects/pyo3-levenshtein

    # Use 'uvx basedpyright' to ensure the correct environment and binary is used.
    # The --output=json flag could be used for more structured output, but simple grep works for error count.

    process = subprocess.run(
        ["uv", "run", "basedpyright", "tests/stub_sample.py"],
        capture_output=True,
        text=True,
        cwd=project_root,  # Ensure pyproject.toml in root is used
    )

    # Assert that basedpyright returned a non-zero exit code (indicating errors)
    assert process.returncode != 0, (
        f"Basedpyright should have found errors, but exited with code 0. Output: {process.stdout}\nErrors: {process.stderr}"
    )

    # Assert that the output contains the expected number of errors
    # basedpyright's output typically ends with a summary like "X errors, Y warnings, Z notes"
    error_summary_line = [line for line in process.stdout.splitlines() if "errors, " in line][-1]

    # Extract the number of errors from the summary line
    # Example: "6 errors, 0 warnings, 0 notes"
    num_errors_str = error_summary_line.split(" errors")[0]
    num_errors = int(num_errors_str)

    # We expect 6 errors as per the intentional incorrect usages
    assert num_errors == 6, (
        f"Expected 6 type errors, but found {num_errors}. Output: {process.stdout}\nErrors: {process.stderr}"
    )

    # Optionally, you can also check for specific error messages if more granular verification is needed.
    # For now, counting errors is sufficient to verify the stub is being used and is correct.
