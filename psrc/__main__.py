#!/usr/bin/env python3
# ======================================================
# Soplang - The Somali Programming Language
# Main entry point for both shell and file execution
# ======================================================

import argparse
import os
import sys

from psrc.core.lexer import Lexer
from psrc.core.parser import Parser
from psrc.core.version import VERSION
from psrc.runtime.interpreter import Interpreter
from psrc.runtime.shell import SoplangShell


def main():
    """
    Main entry point for Soplang.

    This function handles starting the Soplang environment:
    - If run with a filename argument, it executes that file
    - If run with flags, it processes the appropriate action
    - If run without arguments, it launches the interactive shell

    Usage:
        python -m psrc                     # Start interactive shell
        python -m psrc filename.sop        # Execute a Soplang file
        python -m psrc -e 1                # Run example number 1
        python -m psrc -c 'qor("Hello")'  # Execute code snippet
        python -m psrc -v                  # Display version information
    """
    parser = argparse.ArgumentParser(description="Soplang Programming Language")
    parser.add_argument(
        "-v",
        "--version",
        action="store_true",
        help="Display Soplang version information",
    )
    parser.add_argument("-f", "--file", metavar="FILE", help="Execute a Soplang file")
    parser.add_argument(
        "-e", "--example", metavar="N", type=int, help="Run example program number N"
    )
    parser.add_argument(
        "-i",
        "--interactive",
        action="store_true",
        help="Start interactive shell after executing file",
    )
    parser.add_argument(
        "-c", "--command", metavar="CODE", help="Execute Soplang code snippet"
    )
    parser.add_argument("filename", nargs="?", help="Soplang file to execute")

    args = parser.parse_args()

    shell = SoplangShell()

    if args.version:
        print("Soplang - The Somali Programming Language")
        print(f"Version: {VERSION}")
        print("Website: https://www.soplang.org/")
        print("License: MIT")
        return 0

    if args.command:
        shell.execute_code(args.command)
        return 0

    if args.example is not None:
        shell.list_examples("")
        if not shell.last_examples_list:
            return 1

        if args.example < 1 or args.example > len(shell.last_examples_list):
            print(
                f"\033[31mInvalid example number. Choose between 1 and {len(shell.last_examples_list)}\033[0m"
            )
            return 1

        example_file = shell.last_examples_list[args.example - 1]
        example_path = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "examples",
            example_file,
        )
        shell.run_file(example_path)

        if args.interactive:
            shell.run()

        return 0

    filename = args.file or args.filename
    if filename:
        shell.run_file(filename)

        if args.interactive:
            shell.run()

        return 0

    shell.run()
    return 0


if __name__ == "__main__":
    sys.exit(main())
