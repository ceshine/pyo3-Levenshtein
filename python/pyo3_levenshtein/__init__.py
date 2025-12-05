from importlib import metadata

from .pyo3_levenshtein import *  # noqa

try:
    __version__ = metadata.version("pyo3-levenshtein")
except metadata.PackageNotFoundError:
    # This handles the case where the package is imported
    # without being installed (e.g., local script execution)
    __version__ = "unknown"
