#!/usr/bin/env python3
import os
import sys
import subprocess

# This file lives in psrc/; test runner is psrc/tests/runners/run_all_tests.py
script_dir = os.path.dirname(os.path.abspath(__file__))
test_runner = os.path.join(script_dir, "tests", "runners", "run_all_tests.py")

if __name__ == "__main__":
    # Forward all arguments to the actual test runner
    result = subprocess.run([test_runner] + sys.argv[1:])
    sys.exit(result.returncode) 