# FLUX Language Specification v1.0.0

**Fast, Lightweight, Universal eXecution**

This document defines the official FLUX programming language.

---

## 1. File Format

| Property | Value |
|----------|-------|
| Language name | FLUX |
| File extension | `.fx` |
| Encoding | UTF-8 |
| Line endings | LF or CRLF |
| Indentation | Spaces (4 recommended) or tabs |

All FLUX source files use the `.fx` extension. Example: `hello.fx`

---

## 2. Lexical Structure

### 2.1 Comments

Comments begin with `#` and continue to the end of the line.

```flux
# This is a comment
PRINT "Hello"  # inline comment
```

### 2.2 Keywords

Keywords are **case-insensitive** but conventionally written in **UPPERCASE**:

| Category | Keywords |
|----------|----------|
| I/O | `PRINT` |
| Functions | `DEF`, `RETURN`, `LAMBDA` |
| Control flow | `IF`, `ELIF`, `ELSE`, `FOR`, `IN`, `WHILE`, `BREAK`, `CONTINUE`, `PASS` |
| Classes | `CLASS`, `INIT`, `SELF` |
| Modules | `IMPORT`, `FROM`, `AS` |
| Errors | `TRY`, `EXCEPT`, `FINALLY`, `RAISE` |
| Async | `ASYNC`, `AWAIT` |
| Logic | `AND`, `OR`, `NOT` |
| Built-ins | `RANGE`, `TRUE`, `FALSE`, `NONE` |
| Types | `TYPE` |

### 2.3 Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores.

```flux
name
_player
count2
```

### 2.4 Literals

**Strings** — double or single quotes with escape sequences (`\n`, `\t`, `\\`, `\"`, `\'`):

```flux
"Hello World"
'FLUX'
```

**Integers** — decimal whole numbers:

```flux
42
-10
```

**Floats** — decimal numbers with a fractional part:

```flux
3.14
-0.5
```

**Booleans** — `TRUE` or `FALSE`

**None** — `NONE` represents the absence of a value

### 2.5 Operators

| Operator | Meaning |
|----------|---------|
| `=` | Assignment |
| `==` | Equal |
| `!=` | Not equal |
| `<`, `>`, `<=`, `>=` | Comparison |
| `+`, `-`, `*`, `/` | Arithmetic |
| `//` | Floor division |
| `%` | Modulo |
| `**` | Exponentiation |
| `AND`, `OR`, `NOT` | Logical |

### 2.6 Indentation

FLUX uses **significant indentation** (Python-style). After a line ending with `:`, the next line must be indented. Indentation level decreases mark the end of a block.

---

## 3. Data Types

| Type | Description | Example |
|------|-------------|---------|
| `String` | Text | `"hello"` |
| `Integer` | Whole number | `42` |
| `Float` | Decimal number | `3.14` |
| `Boolean` | True or false | `TRUE` |
| `None` | No value | `NONE` |
| `Array` | Ordered collection | `[1, 2, 3]` |
| `Dict` | Key-value map | `{"a": 1}` |
| `Function` | Callable | `DEF f(): ...` |
| `Object` | Class instance | `Player("Aria")` |

---

## 4. Statements

### 4.1 Print

```flux
PRINT "Hello World"
PRINT name
PRINT 1 + 2
```

### 4.2 Variables

```flux
name = "Alex"
age = 20
pi = 3.14
active = TRUE
```

### 4.2.1 Ask / Answer

`ASK` prompts on the console and stores whatever the user types (auto-converted
to Integer, Float, or String) in the variable `ANSWER`. Both of the following
are equivalent — the `=` is optional:

```flux
ASK "What is your name?"
PRINT "Hello, " + ANSWER

ASK = "How old are you?"
PRINT ANSWER + 1
```

`ANSWER` is overwritten every time `ASK` runs, so read it before the next `ASK`.

### 4.3 Functions

```flux
DEF greet(name):
    RETURN "Hello " + name

message = greet("World")
```

Parameters may include type hints and defaults:

```flux
DEF connect(host, port: Integer = 8080):
    PRINT host
```

### 4.4 Conditions

```flux
IF age >= 18:
    PRINT "Adult"
ELIF age >= 13:
    PRINT "Teen"
ELSE:
    PRINT "Minor"
```

### 4.5 Loops

```flux
FOR i IN RANGE(10):
    PRINT i

WHILE running:
    running = FALSE
```

`RANGE(n)` produces integers from `0` to `n-1`.  
`RANGE(start, end)` produces integers from `start` to `end-1`.

### 4.6 Classes

```flux
CLASS Player:
    DEF INIT(name):
        self.name = name

    DEF greet(self):
        RETURN "Hello " + self.name

hero = Player("Aria")
PRINT hero.greet()
```

`INIT` is the constructor. `SELF` refers to the current instance inside methods.

### 4.7 Modules

```flux
IMPORT math
IMPORT io AS filesystem

FROM sys IMPORT version, platform
```

### 4.8 Error Handling

```flux
TRY:
    result = risky_operation()
EXCEPT Exception AS e:
    PRINT e
FINALLY:
    PRINT "Done"
```

Raise an exception:

```flux
RAISE "Something went wrong"
```

---

## 5. Expressions

Expressions produce values. Operator precedence (highest to lowest):

1. Parentheses, indexing, member access, calls
2. Unary `-`, `NOT`
3. `**`
4. `*`, `/`, `//`, `%`
5. `+`, `-`
6. Comparisons
7. `AND`
8. `OR`

---

## 6. Standard Library (Built-in)

### Global Functions

| Function | Description |
|----------|-------------|
| `len(x)` | Length of string, array, or dict |
| `str(x)` | Convert to string |
| `int(x)` | Convert to integer |
| `float(x)` | Convert to float |
| `type(x)` | Get type name |
| `RANGE(n)` | Create integer range |

### Modules

| Module | Description |
|--------|-------------|
| `math` | `abs`, `min`, `max` |
| `io` | `read_file`, `write_file` |
| `sys` | `version`, `platform` |
| `FXwindows` | Windows, buttons, labels, and other UI widgets — see [`libraries/FXwindows/README.md`](../libraries/FXwindows/README.md) |

Future modules: `graphics`, `net`, `db`

---

## 7. Package Format

FLUX packages use `flux.toml`:

```toml
[project]
name = "MyLibrary"
version = "1.0.0"
description = "A FLUX library"
authors = ["Developer"]
flux_version = "1.0.0"

[dependencies]
```

Directory layout:

```
MyLibrary/
├── flux.toml
├── src/
│   └── library.fx
└── README.md
```

Install: `fx install MyLibrary`  
Import: `IMPORT graphics`

---

## 8. Execution Model

### Development Mode

```bash
fx run hello.fx
```

Runs source through the FLUX interpreter for instant feedback.

### Production Mode

```bash
fx build hello.fx
```

Compiles to a native executable (e.g., `hello.exe` on Windows).  
*Available in Phase 3 (LLVM backend).*

---

## 9. Error System

| Error Type | Description |
|------------|-------------|
| SyntaxError | Invalid FLUX syntax |
| TypeError | Operation on wrong type |
| NameError | Undefined variable |
| IndexError | Invalid array/string index |
| KeyError | Missing dictionary key |
| Exception | User-raised exception |

Errors report line and column when available.

---

## 10. CLI Reference

```bash
flux run hello.fx       # Run program
flux build hello.fx     # Compile program
flux create MyProject   # Create project
flux install Package    # Install package
flux remove Package     # Remove package
flux update             # Update packages
flux --version          # Show version
flux --help             # Show help
```

Both `flux` and `fx` are official command names.

---

## 11. Version

**FLUX 1.0.0** — Fast, Lightweight, Universal eXecution
