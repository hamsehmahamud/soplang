# Soplang Python Implementation — Full Analysis

> Reference document for the Rust rewrite. Describes every component of the Python implementation (`psrc/`) in detail: architecture, data structures, algorithms, design decisions, and known limitations.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Data Flow](#data-flow)
3. [Module Breakdown](#module-breakdown)
   - [tokens.py](#1-psr-coretokenspy--token-types)
   - [lexer.py](#2-psrccorelexerpy--lexer)
   - [ast.py](#3-psr-coreastpy--ast)
   - [parser.py](#4-psr-coreparserpy--parser)
   - [errors.py](#5-psrcutilserrorspy--error-system)
   - [interpreter.py](#6-psrcruntime-interpreterpy--interpreter)
   - [builtins.py](#7-psr-stdlibbuiltinspy--standard-library)
   - [shell.py](#8-psr-runtimeshellpy--repl--shell)
4. [Type System](#type-system)
5. [Scoping Model](#scoping-model)
6. [Control Flow Signals](#control-flow-signals)
7. [Class System](#class-system)
8. [Import System](#import-system)
9. [Error Message System](#error-message-system)
10. [Known Limitations](#known-limitations)
11. [Rust Translation Reference](#rust-translation-reference)

---

## Architecture Overview

The Python implementation is a **tree-walking interpreter**. There is no bytecode, no compilation step, and no intermediate representation. The pipeline is:

```
source (.sop)  →  Lexer  →  [Token]  →  Parser  →  ASTNode  →  Interpreter  →  output
```

All three stages are hand-written with no third-party parsing libraries.

```
psrc/
├── core/
│   ├── tokens.py       TokenType enum (~50 variants)
│   ├── lexer.py        Character-by-character lexer
│   ├── parser.py       Recursive descent parser (~1 100 lines)
│   └── ast.py          Generic ASTNode + NodeType enum
├── runtime/
│   ├── interpreter.py  Tree-walking interpreter (~970 lines)
│   ├── main.py         run_soplang_file() helper
│   └── shell.py        REPL (prompt_toolkit)
├── stdlib/
│   └── builtins.py     All built-in functions and methods
└── utils/
    └── errors.py       Error hierarchy + Somali error messages
```

---

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  source string                                                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                     Lexer.tokenize()
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│  List[Token]   { type: TokenType, value: Any, line, col }      │
└────────────────────────────┬────────────────────────────────────┘
                             │
                     Parser.parse()
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│  ASTNode(PROGRAM)                                               │
│    ├── ASTNode(VARIABLE_DECLARATION, value="x")                │
│    ├── ASTNode(FUNCTION_DEFINITION, value="salaam")            │
│    └── ASTNode(IF_STATEMENT)                                   │
│          ├── ASTNode(BINARY_OPERATION, value=">")              │
│          └── ...                                               │
└────────────────────────────┬────────────────────────────────────┘
                             │
                  Interpreter.interpret()
                             │
                             ▼
                     side effects + output
```

---

## Module Breakdown

### 1. `psrc/core/tokens.py` — Token types

**Class:** `TokenType(Enum)`

All tokens are a single flat enum. There are roughly four groups:

#### Keywords (Somali → English equivalent)
| Token | Keyword | Meaning |
|-------|---------|---------|
| `DOOR` | `door` | dynamic variable declaration (`var`/`let`) |
| `MADOOR` | `madoor` | constant variable declaration (`const`) |
| `HAWL` | `hawl` | function definition (`func`/`def`) |
| `CELI` | `celi` | return statement |
| `qor` | `qor` | print to stdout |
| `GELIN` | `gelin` | read user input |
| `HADDII` | `haddii` | if |
| `HADDII_KALE` | `haddii_kale` | else if |
| `UGUDAMBEYN` | `ugudambeyn` | else |
| `DOORO` | `dooro` | switch |
| `XAALAD` | `xaalad` | case |
| `kuceli` | `kuceli` | for loop |
| `INTAY` | `intay` | while loop |
| `JOOJI` | `jooji` | break |
| `soco` | `soco` | continue |
| `ISKU_DAY` | `isku_day` | try |
| `QABO` | `qabo` | catch |
| `KA_KEEN` | `ka_keen` | import |
| `FASALKA` | `fasalka` | class definition |
| `KA_DHAXAL` | `ka_dhaxal` | extends (inheritance) |
| `CUSUB` | `cusub` | new (object instantiation) |
| `NAFTA` | `nafta` | self / this |

#### Static type keywords
| Token | Keyword | Type |
|-------|---------|------|
| `abn` | `abn` | integer |
| `JAJAB` | `jajab` | float |
| `QORAAL` | `qoraal` | string |
| `BOOL` | `bool` | boolean |
| `teed` | `teed` | list |
| `WALAX` | `walax` | object/dict |

#### Literal and identifier tokens
`IDENTIFIER`, `NUMBER`, `STRING`, `TRUE` (`run`), `FALSE` (`been`), `NULL` (`null`)

#### Operator and structural tokens
`PLUS`, `MINUS`, `STAR`, `SLASH`, `MODULO`, `EQUAL` (`==`), `NOT_EQUAL`, `GREATER`, `LESS`, `GREATER_EQUAL`, `LESS_EQUAL`, `AND` (`&&`), `OR` (`||`), `NOT` (`!`), `ASSIGN` (`=`), `COMMA`, `COLON`, `SEMICOLON`, `LEFT_PAREN`, `RIGHT_PAREN`, `LEFT_BRACE`, `RIGHT_BRACE`, `LEFT_BRACKET`, `RIGHT_BRACKET`, `DOT`, `EOF`

> **Note:** The `ASSIGN` token (`=`) is separate from `EQUAL` (`==`). The parser disambiguates them by context.

---

### 2. `psrc/core/lexer.py` — Lexer

**Class:** `Lexer(source_code: str)`  
**Main method:** `tokenize() -> List[Token]`

#### Algorithm
- Maintains a single character cursor (`position`, `current_char`)
- Tracks `line` and `column` for error reporting
- `next_token()` dispatches on the current character
- `tokenize()` calls `next_token()` in a loop until `EOF`

#### Token recognition
| Input | Handler |
|-------|---------|
| whitespace | `skip_whitespace()` — advance, discard |
| `//` | `skip_comment()` — advance to end of line |
| `/* ... */` | `skip_comment()` — advance to `*/`, raises `LexerError("unterminated_comment")` if not closed |
| alpha or `_` | `tokenize_identifier()` — reads alphanumeric+`_`, checks `KEYWORDS` dict |
| digit | `tokenize_number()` — reads digits, detects `.` for float, returns `int` or `float` |
| `"` or `'` | `tokenize_string()` — reads until matching quote, raises `LexerError("unterminated_string")` |
| `>`, `<`, `!`, `=` | reads next char for two-char tokens (`>=`, `<=`, `!=`, `==`) |
| `&` | expects second `&` for `&&`, else `LexerError` |
| `\|` | expects second `\|` for `\|\|`, else `LexerError` |
| anything else | raises `LexerError("unexpected_char", char=...)` |

#### Error conditions
- `unexpected_char` — unrecognised character
- `unterminated_string` — string literal spans EOF
- `unterminated_comment` — block comment spans EOF

---

### 3. `psrc/core/ast.py` — AST

#### `NodeType(Enum)` — 20 variants

| Category | Node types |
|----------|-----------|
| Root | `PROGRAM`, `BLOCK` |
| Declarations | `VARIABLE_DECLARATION`, `FUNCTION_DEFINITION`, `CLASS_DEFINITION`, `IMPORT_STATEMENT` |
| Control flow | `IF_STATEMENT`, `SWITCH_STATEMENT`, `LOOP_STATEMENT`, `WHILE_STATEMENT`, `TRY_CATCH`, `BREAK_STATEMENT`, `CONTINUE_STATEMENT`, `RETURN_STATEMENT` |
| Expressions | `BINARY_OPERATION`, `UNARY_OPERATION`, `LITERAL`, `IDENTIFIER`, `FUNCTION_CALL`, `ASSIGNMENT` |
| Data structures | `LIST_LITERAL`, `OBJECT_LITERAL`, `PROPERTY_ACCESS`, `METHOD_CALL`, `INDEX_ACCESS` |

#### `ASTNode` — generic node structure

```python
class ASTNode:
    type: NodeType
    value: Any           # overloaded: var name, function name, operator, literal value
    children: List[ASTNode]
    var_type: TokenType  # only on VARIABLE_DECLARATION (static type annotation)
    is_constant: bool    # only on VARIABLE_DECLARATION (madoor)
    line: int            # source line number
    position: int        # source column
```

**Key design choice:** There is one generic node class. All nodes share the same fields, most of which are `None` for any given node type. This is flexible but loses static safety — the interpreter must know which fields are valid for each node type.

#### How `children` is used per node type

| Node type | `value` | `children` layout |
|-----------|---------|-------------------|
| `VARIABLE_DECLARATION` | variable name | `[expr]` |
| `FUNCTION_DEFINITION` | function name | `[param_idents..., body_stmts...]` |
| `FUNCTION_CALL` | function name | `[args...]` |
| `IF_STATEMENT` | — | `[condition, then_stmts..., elif_nodes..., else_block?]` |
| `SWITCH_STATEMENT` | — | `[switch_expr, case_blocks...]` |
| `LOOP_STATEMENT` | loop var name | `[start, end, (step?), body_stmts...]` |
| `WHILE_STATEMENT` | — | `[condition, body_stmts...]` |
| `BINARY_OPERATION` | operator string | `[left, right]` |
| `UNARY_OPERATION` | operator string | `[expr]` |
| `ASSIGNMENT` | — | `[target, value]` |
| `PROPERTY_ACCESS` | property name | `[object]` |
| `METHOD_CALL` | method name | `[object, args...]` |
| `INDEX_ACCESS` | — | `[object, index]` |
| `LIST_LITERAL` | — | `[elements...]` |
| `OBJECT_LITERAL` | — | `[property_nodes...]` where each property node is `LITERAL(key, [value])` |
| `CLASS_DEFINITION` | `name` or `(name, parent)` | `[body_stmts...]` |
| `TRY_CATCH` | error var name | `[try_block, catch_block]` |
| `IMPORT_STATEMENT` | filename string | `[]` |
| `RETURN_STATEMENT` | — | `[expr]` or `[]` |
| `BREAK_STATEMENT` | — | `[]` |
| `CONTINUE_STATEMENT` | — | `[]` |

---

### 4. `psrc/core/parser.py` — Parser

**Class:** `Parser(tokens: List[Token])`  
**Main method:** `parse() -> ASTNode(PROGRAM)`

#### Operator precedence (lowest to highest)

```
logical         parse_logical_expression      &&  ||
comparison      parse_comparison_expression   ==  !=  >  <  >=  <=
additive        parse_expression              +  -
multiplicative  parse_term                    *  /  %
unary           parse_factor                  -  !  +  (unary)
postfix         parse_postfix                 . []  ()  (method/index/call chains)
primary         parse_primary                 literals, identifiers, ( expr ), [ list ], { object }
```

#### Statement dispatch (`parse_statement`)
The parser dispatches on the current token type:

| Token | Parses |
|-------|--------|
| `HADDII` | if / elseif / else chain |
| `DOORO` | switch / case / default |
| `DOOR` | dynamic variable declaration |
| `MADOOR` | constant declaration (with optional type) |
| `abn/jajab/qoraal/bool/teed/walax` | static variable declaration |
| `HAWL` | function definition |
| `CELI` | return statement |
| `qor` | print statement (parsed as FUNCTION_CALL) |
| `kuceli` | for loop |
| `INTAY` | while loop |
| `JOOJI` | break |
| `soco` | continue |
| `ISKU_DAY` | try/catch |
| `KA_KEEN` | import |
| `FASALKA` | class definition |
| `LEFT_BRACE` | anonymous block |
| `IDENTIFIER` | assignment, function call, property chain, method call |

#### For loop syntax
```
kuceli (i 1 ilaa 10) { ... }           # i from 1 to 10
kuceli (i 1 ilaa 10 :: 2) { ... }      # i from 1 to 10 step 2
```
The `ilaa` separator is parsed as an identifier token (not a keyword).

#### Class definition syntax
```
fasalka Xayawaan { ... }
fasalka Ey ka_dhaxal Xayawaan { ... }
```
When inheritance is present, `node.value` is a tuple `(class_name, parent_name)`.

#### Known parser issue
`execute_assignment()` method appears in `parser.py` but is actually dead code — the real assignment execution is in `interpreter.py`. This is a leftover artefact.

---

### 5. `psrc/utils/errors.py` — Error system

#### Hierarchy

```
Exception
└── SoplangError
    ├── LexerError(error_code, line, col, **kwargs)
    ├── ParserError(error_code, token, line, col, **kwargs)
    ├── TypeError(error_code, line, col, **kwargs)
    ├── ValueError(message, line, col)
    ├── NameError(name, line, col)
    ├── ImportError(error_code, line, col, **kwargs)
    └── RuntimeError(error_code, line, col, **kwargs)
```

#### Control flow signals (not errors)

```
Exception
├── BreakSignal          raised by jooji, caught by loop executor
├── ContinueSignal       raised by soco, caught by loop executor
└── ReturnSignal(value)  raised by celi, caught by function executor
```

Signals use Python's exception mechanism purely for non-local exit — they are always caught and never displayed to the user.

#### `ErrorMessageManager`

Centralised Somali error message templates, keyed by error code:

| Category | Example code | Somali template |
|----------|-------------|-----------------|
| Lexer | `unexpected_char` | `Xaraf aan la filayn: {char}` |
| Lexer | `unterminated_string` | `Qoraal aan la dhammaystirin` |
| Parser | `expected_token` | `Waxaa la filayay {expected}, laakiin waxaa la helay {found}` |
| Parser | `unexpected_token` | `Calaamad aan la filayn: {token}` |
| Type | `type_mismatch` | `'{var_name}' waa {expected_type} laakin qiimaheeda '{value}' ma ahan {expected_type}` |
| Runtime | `undefined_variable` | `Doorsame aan la qeexin: '{name}'` |
| Runtime | `division_by_zero` | `Ma suurtogali karto qeybinta eber` |
| Runtime | `constant_reassignment` | `Ma bedeli kartid qiimaha doorsamaha madoor '{name}'...` |
| Import | `file_not_found` | `Faylka '{module}' ma helin` |

Final error format: `Khalad {type}: {message} sadar {line}, goobta {col}`

---

### 6. `psrc/runtime/interpreter.py` — Interpreter

**Class:** `Interpreter`  
**~970 lines**

#### State

```python
self.variables: dict          # all variables in scope (single flat dict)
self.variable_types: dict     # var name → TokenType (for static-typed vars)
self.constant_variables: set  # var names that are madoor (const)
self.functions: dict          # name → callable or {"params", "body", "closure_vars"}
self.list_methods: dict       # method name → callable
self.object_methods: dict
self.string_methods: dict
self.classes: dict            # class name → {"name", "parent", "body", "methods"}
self.call_stack: list         # stack of function names (for future use)
```

#### Execute pipeline

```
interpret(root: ASTNode)
  └── execute(stmt) for stmt in root.children
        ├── execute_var_declaration
        ├── define_function
        ├── execute_function_call
        ├── execute_if_statement
        ├── execute_switch_statement
        ├── execute_loop_statement        (kuceli / for)
        ├── execute_while_statement
        ├── execute_block
        ├── execute_import_statement
        ├── execute_try_catch
        ├── execute_class_definition
        └── execute_assignment
```

#### Evaluate pipeline

```
evaluate(expr: ASTNode) → Any
  ├── LITERAL         → raw Python value (int/float/str/bool/None)
  ├── IDENTIFIER      → variables[name]
  ├── BINARY_OPERATION→ evaluate left + right, apply operator
  ├── UNARY_OPERATION → -x  or  !x
  ├── FUNCTION_CALL   → look up in self.functions, dispatch
  ├── METHOD_CALL     → evaluate receiver, dispatch to method dict
  ├── PROPERTY_ACCESS → evaluate receiver dict, look up key
  ├── INDEX_ACCESS    → evaluate receiver list, look up index
  ├── LIST_LITERAL    → [evaluate(e) for e in children]
  └── OBJECT_LITERAL  → {prop.value: evaluate(prop.children[0]) for prop in children}
```

#### Binary operators

| Operator | Behaviour |
|----------|-----------|
| `+` | numeric add or string concatenation (auto-converts right side to string if left is string) |
| `-` `*` `/` `%` | numeric only; `/` and `%` raise `RuntimeError("division_by_zero")` |
| `==` `!=` `>` `<` `>=` `<=` | standard comparison |
| `&&` `\|\|` | short-circuit logical |

#### Function execution
1. Evaluate all argument expressions
2. `old_vars = self.variables.copy()` — save current scope
3. Bind params to args in `self.variables`
4. Execute body statements; catch `ReturnSignal` for the return value
5. `self.variables = old_vars` — restore scope

> **Limitation:** This is not true lexical scoping. Inner functions see the scope at call time, not at definition time. No closures.

#### Method dispatch
When evaluating `METHOD_CALL`:
1. Evaluate the receiver
2. Dispatch to `self.list_methods[method]`, `self.object_methods[method]`, or `self.string_methods[method]` based on `isinstance` check
3. For class instances (dicts with `__class__`): look up the method in the class body and call it with `nafta=instance`

---

### 7. `psrc/stdlib/builtins.py` — Standard library

**Class:** `SoplangBuiltins` (all `@staticmethod`)

#### Built-in functions (14)

| Soplang name | Python equivalent | Notes |
|-------------|-------------------|-------|
| `qor` | `print()` | converts value to string first via `qoraal()` |
| `gelin` | `input()` | reads a line from stdin |
| `nooc` | `type()` | returns Somali type name as string |
| `abn` | `int()` | converts to integer |
| `jajab` | `float()` | converts to float |
| `qoraal` | `str()` | converts to string; booleans → `"run"`/`"been"` |
| `bool` | `bool()` | converts to boolean |
| `teed` | `list()` | creates a list from args |
| `walax` | `dict()` | creates an object from kwargs |
| `daji` | `math.floor()` | floor |
| `kor` | `math.ceil()` | ceiling |
| `dherer` | `len()` | works on list, string, object |
| `xul` | `random` | 0 args → float, 1 arg (list) → choice, 2 args → randint/uniform |
| `baaxad` | `range()` | 1–3 args: stop / start+stop / start+stop+step, returns list |

#### List methods (12)

| Method | Soplang | Behaviour |
|--------|---------|-----------|
| `kasaar` | pop | removes + returns last item |
| `dherer` | length | returns len |
| `kudar` | concat/push | if arg is list: new concatenated list; else push in-place |
| `leeyahay` | contains | `item in list` |
| `nuqul` | copy | shallow copy |
| `nadiifi` | clear | clears in-place |
| `rog` | reverse | reverses in-place |
| `habee` | sort | sorts in-place ascending |
| `shaandhee` | filter | returns new list (accepts a Soplang function as arg) |
| `jar` | slice | `list[start:end]` with negative index support |
| `aaddin` | map | returns new list (accepts a Soplang function as arg) |
| `muuji` | indexOf | returns index or `None` |

#### Object methods (8)

| Method | Behaviour |
|--------|-----------|
| `fure` | returns list of keys |
| `leeyahay` | key membership check |
| `tir` | deletes key |
| `kudar` | merges two objects into new object |
| `nuqul` | shallow copy |
| `nadiifi` | clears in-place |
| `qiime` | returns list of values |
| `lamaane` | returns list of `[key, value]` pairs |

#### String methods (12)

| Method | Behaviour |
|--------|-----------|
| `qeybi` | split by delimiter |
| `leeyahay` | substring membership |
| `dhamaad` | endswith |
| `bilow` | startswith |
| `beddel` | replace first occurrence |
| `beddel_dhammaan` | replace all occurrences |
| `kudar` | join list of strings |
| `jar` | slice `[start:end]` |
| `xarafaha_weyn` | uppercase |
| `xarfaha_yaryar` | lowercase |
| `masax` | strip whitespace |
| `raadi` | find index of substring (returns -1 if not found) |

---

### 8. `psrc/runtime/shell.py` — REPL / Shell

**Class:** `SoplangShell`

- Uses `prompt_toolkit` for readline, history, tab completion, syntax colouring
- `execute_code(source)` — runs `Lexer → Parser → Interpreter` pipeline inline
- `run_file(path)` — reads `.sop` file, passes to `execute_code`
- `run()` — interactive loop: prompt, read line, execute, print result
- Error output: strips Python tracebacks, replaces Python module paths with Soplang-friendly messages, prints Somali error text in red
- `list_examples()` — scans `examples/` directory, presents numbered list
- `run_file()` + `execute_import_statement()` — share the same pipeline

---

## Type System

Soplang has a **hybrid** type system. Variables can be declared either way:

```
door x = 10          // dynamic: x can be reassigned to any type
abn y = 10           // static: y is always abn (integer)
madoor PI = 3.14     // constant dynamic
madoor jajab E = 2.71 // constant static
```

#### Static type checking

Checking happens at **assignment time** in `validate_type()`:

| Keyword | Python isinstance check |
|---------|------------------------|
| `abn` | `isinstance(value, (int, float))` |
| `jajab` | `isinstance(value, (int, float))` |
| `qoraal` | `isinstance(value, str)` |
| `bool` | `isinstance(value, bool)` |
| `teed` | `isinstance(value, list)` |
| `walax` | `isinstance(value, dict)` |

There is no static type inference and no compile-time checking — types are only enforced at runtime.

#### Runtime values

Python native types are used directly as Soplang values:

| Soplang type | Python type |
|-------------|-------------|
| `abn` | `int` |
| `jajab` | `float` |
| `qoraal` | `str` |
| `bool` | `bool` |
| `teed` | `list` |
| `walax` | `dict` |
| `maran` (null) | `None` |
| function | `dict` `{"params", "body"}` or Python `callable` |
| class instance | `dict` with `"__class__"` key |

---

## Scoping Model

The interpreter uses a **single flat dictionary** for variables. Function calls save and restore the full dict:

```python
old_vars = self.variables.copy()   # save
self.variables[param] = arg        # bind params
# ... execute body ...
self.variables = old_vars           # restore
```

**Consequences:**
- No true lexical scoping
- No closures
- Nested function definitions do not capture their enclosing scope
- `ka_keen` (import) executes in the same flat scope — all names from the imported file are merged into the caller's namespace

---

## Control Flow Signals

Break, continue, and return are implemented using Python exceptions as non-local exit:

```
kuceli (i 0 ilaa 10) {
    haddii (i == 5) { jooji }     ← raises BreakSignal
}
```

```
execute_loop_statement
  └── execute(body_stmt)
        └── ... (nested calls) ...
              └── execute(BREAK_STATEMENT)
                    └── raise BreakSignal()   ← propagates up
execute_loop_statement catches BreakSignal, exits loop
```

`ReturnSignal(value)` carries the return value up through arbitrarily deep nested function bodies.

---

## Class System

```
fasalka Xayawaan {
    hawl dhaw(nafta, magac) {
        nafta.magac = magac
    }
    hawl hadal(nafta) {
        qor("Xayawaanku wuxuu yidhi: " + nafta.magac)
    }
}
door h = cusub Xayawaan("Libaax")
h.hadal()
```

#### Implementation details

- Class definition stored as: `{"name", "parent", "body": List[ASTNode], "methods": {name: ASTNode}}`
- `cusub ClassName(args)`: creates a Python `dict` instance `{"__class__": "ClassName"}`; finds and calls `dhaw` (constructor) with `nafta=instance`
- Method calls on an instance: look up `__class__`, find method in `self.classes[class_name]["methods"]`, execute with `nafta` bound to the instance
- Inheritance (`ka_dhaxal`): if method not found in current class, walk up `parent` chain
- Property access/set: `instance["prop"]` / `instance["prop"] = value` directly on the dict

---

## Import System

```
ka_keen "math_utils.sop"
```

1. Resolve the path relative to the currently executing file
2. Read the file contents
3. Run the full `Lexer → Parser → Interpreter` pipeline on the file
4. All names defined in the imported file land in the **current interpreter's `self.variables`**

There is no module namespace, no `as` aliasing, and no circular import protection.

---

## Error Message System

All user-facing errors are in Somali. The format is:

```
Khalad {type}: {message} sadar {line}, goobta {col}
```

Examples:
```
Khalad runtime: Doorsame aan la qeexin: 'x' sadar 3, goobta 5
Khalad nooc: 'age' waa qoraal laakin qiimaheeda '25' ma ahan qoraal sadar 7, goobta 1
Khalad lexer: Xaraf aan la filayn: '@' sadar 1, goobta 12
```

---

## Known Limitations

| # | Limitation | Impact |
|---|-----------|--------|
| 1 | No closures — inner functions don't capture enclosing scope | Cannot write factory functions or callbacks with closed-over state |
| 2 | Flat scoping — variable names collide across call frames | Variables from outer scope are visible inside functions; modifying them affects the outer scope |
| 3 | Flat import — no namespacing | Imported files can silently overwrite existing variables |
| 4 | No circular import protection | `ka_keen` can loop infinitely |
| 5 | Single-threaded | No concurrency primitives |
| 6 | `abn` and `jajab` share `(int, float)` in isinstance checks | `abn x = 3.14` does not raise an error |
| 7 | `execute_assignment` in parser.py | Dead code; confusing for maintenance |
| 8 | `SoplangError` subclasses shadow Python builtins | `RuntimeError`, `TypeError`, `ValueError`, `ImportError`, `NameError` are redefined |
| 9 | Function missing-argument handling | Missing args default to `None` silently |
| 10 | No tail-call optimisation | Deep recursion hits Python's stack limit |

---

## Rust Translation Reference

| Python concept | Rust equivalent |
|----------------|----------------|
| `TokenType(Enum)` | `enum TokenType` (derived `Clone`, `Debug`, `PartialEq`) |
| `Token` dataclass | `struct Token { kind: TokenType, lexeme: String, line: usize, col: usize }` |
| Generic `ASTNode` | Typed `enum Expr` + `enum Stmt` |
| Python dynamic value | `enum Value { Int(i64), Float(f64), Str(String), Bool(bool), List(...), Object(...), Null, Function(...) }` |
| `list` runtime value | `Rc<RefCell<Vec<Value>>>` |
| `dict` runtime value | `Rc<RefCell<IndexMap<String, Value>>>` |
| Flat `self.variables` | `struct Env { vars: HashMap<String,Value>, parent: Option<Rc<RefCell<Env>>> }` |
| Python exceptions for signals | `enum Signal { None, Break, Continue, Return(Value) }` returned as `Ok(Signal)` |
| `SoplangError` hierarchy | `enum SoplangError { Lexer{msg,line,col}, Parser{...}, Runtime{...}, Type{...}, Import{...} }` |
| `raise LexerError(...)` | `return Err(SoplangError::Lexer { ... })` |
| `try/except ReturnSignal` | `match exec_stmt(...) { Ok(Signal::Return(v)) => ... }` |
| `prompt_toolkit` | `rustyline` crate |
