# Py++

Py++ is a small Python-based programming language with Python-like syntax and compile-time macro support. The language is parsed into a custom AST and compiled into a Python AST for execution.

## Overview

Py++ supports:

- function definitions with `func`
- class definitions with `class`
- compile-time macros with `macro` and `$macro[...]`
- `use` and `from ... use` imports for Python and Py++ modules
- `if`, `while`, `for`, `match`, `try/except`, `with` and other standard control flow
- `when` hooks for field assignment interception in classes
- annotations, generators, comprehensions, f-strings, and more

The implementation lives in `pyplusplus/`:

- `pyplusplus/parser.py` — parses Py++ source into AST nodes
- `pyplusplus/compiler.py` — compiles AST into Python AST for execution
- `pyplusplus/cli.py` — command-line entry point for running Py++ files
- `pyplusplus/ast_nodes.py` — AST node definitions

## Installation

1. Create or activate your Python environment.
2. Install the package for development:

```bash
python -m pip install -U pip
python -m pip install -e .[test]
```

## Running tests

Run the project tests with:

```bash
pytest
```

## Using the CLI

Use the CLI to parse or execute Py++ files.

```bash
python -m pyplusplus.cli path/to/file.pypp --run
```

Example:

```bash
python -m pyplusplus.cli examples/example.pypp --run
```

If you only want to inspect the compiled Python AST:

```bash
python -m pyplusplus.cli examples/example.pypp --dump-ast
```

## Writing Py++ code

Example `examples/macros.pypp` features:

```pypp
print("Hello, World!");

macro printer_if[var: Identifier, expected: Expr, text: Expr] {
    if (var == expected) {
        print(text);
    }
}

macro do_twice[code: Stmt] {
    code;
    code;
}

a = 1;
b = 2;

$printer_if[a, 1, "a is 1!"];
$do_twice[{ print("twice"); }];
```

Macro calls are written as `$name[...]`, and macro bodies are expanded at compile time.

Importing modules:

- `use "module.pypp" as mod;`
- `from "module.pypp" use func;`
- `use "typing" as t;`

## `when` hooks

Py++ supports `when` hooks inside classes to intercept and validate field assignment. A `when` block runs whenever the named field is assigned, allowing custom logic before the assignment completes.

Example:

```pypp
class Counter {
    when value(new_value) {
        if (new_value < 0) {
            raise ValueError("Value cannot be negative");
        }
        return new_value;
    }
}

counter = Counter();
counter.value = 1;
```

## VS Code support

A minimal VS Code syntax extension is available at:

https://github.com/Pypp-lang/vscode-pyplusplus-extension

To enable syntax highlighting for `.pypp` files in VS Code, install the extension or add:

```json
{
  "files.associations": {
    "*.pypp": "pyplusplus"
  }
}
```

## Project structure

- `examples/` — example Py++ programs
- `libs_pypp/` — reusable Py++ standard library helpers
- `pyplusplus/` — language implementation
- `tests/` — parser and compiler tests

## Notes

Py++ is designed for experimentation with language features on top of Python. The compiler generates Python AST that can be executed directly using Python's built-in `compile` and `exec` functions.
