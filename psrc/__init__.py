#!/usr/bin/env python3
"""
Soplang: The Somali Programming Language
=======================================

Soplang is a modern, semantically rich programming language designed with Somali
keywords, offering an accessible entry point for Somali speakers in the world of
programming while maintaining professional development capabilities.

Key Features:
- Intuitive Somali language keywords that lower barriers to entry
- Strong type system with both static and dynamic typing options
- First-class functions with closures and recursion support
- Rich data structures including lists and associative objects
- Object-oriented programming with inheritance
- Comprehensive error handling with meaningful Somali error messages
- Interactive REPL environment for rapid prototyping and learning

Soplang bridges linguistic accessibility with computing concepts, making
programming more inclusive while maintaining a strong technical foundation.
"""

# Core language components
from psrc.core.lexer import Lexer, Token
from psrc.core.parser import Parser
from psrc.core.tokens import TokenType
from psrc.core.ast import ASTNode, NodeType

# Runtime components
from psrc.runtime.interpreter import Interpreter
from psrc.runtime.shell import SoplangShell

# Utilities and error handling
from psrc.utils.errors import (
    SoplangError, LexerError, ParserError, RuntimeError,
    TypeError, ImportError, BreakSignal, ContinueSignal, ReturnSignal
)

# Standard library
from psrc.stdlib.builtins import get_builtin_functions, get_list_methods, get_object_methods

# Make src a proper Python package
