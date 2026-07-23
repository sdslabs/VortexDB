# This file is like a master test. Runs all the examples
# Not exactly the purpose of the examples dir,
# but helps in checking if any code updates haven't broken the API

from pathlib import Path
import subprocess
import pytest

# Didn't know I could do this with pytest, so cool
# Just run: pytest ./all.py -v

EXAMPLES_DIR = Path(__file__).parent
example_files = sorted(EXAMPLES_DIR.glob("*_usage.py"))


@pytest.mark.parametrize(
    "script_path",
    example_files,
    ids=lambda p: p.stem,
)
def test(script_path):
    """Run all example scripts to check if they crash or not"""
    result = subprocess.run(
        ["python3", str(script_path)], capture_output=True, text=True
    )
    assert result.returncode == 0, (
        f"Script {script_path} failed with stderr:\n{result.stderr}"
    )
