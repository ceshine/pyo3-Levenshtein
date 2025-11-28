#!/bin/bash
# Wrapper script to run cargo test with automatic Python detection

# Get the directory containing this script (project root)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_PYTHON="$SCRIPT_DIR/.venv/bin/python"

# Check if venv Python exists
if [ ! -f "$VENV_PYTHON" ]; then
    echo "Error: Virtual environment not found at $VENV_PYTHON" >&2
    echo "Please create a virtual environment first: uv venv" >&2
    exit 1
fi

# Set PYO3_PYTHON for the build phase
export PYO3_PYTHON="$VENV_PYTHON"

# Query Python for its library directory and home
# For venvs, we need the base_prefix (actual Python install), not the venv prefix
PYTHON_LIBDIR=$("$VENV_PYTHON" -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))" 2>/dev/null)
PYTHON_HOME=$("$VENV_PYTHON" -c "import sys; print(sys.base_prefix)" 2>/dev/null)

# Set environment variables for Python runtime
export LD_LIBRARY_PATH="$PYTHON_LIBDIR:$LD_LIBRARY_PATH"
export PYTHONHOME="$PYTHON_HOME"


# Run cargo test (test-runner.sh will handle runtime environment)
exec cargo test "$@"
