#!/usr/bin/env python3
import sys
import os

# Add the project root directory to the Python path (psrc/tests/runners -> ../../..)
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../../..")))

# Import the test module
from psrc.tests.test_parser import TestParser
import unittest

if __name__ == "__main__":
    unittest.main() 