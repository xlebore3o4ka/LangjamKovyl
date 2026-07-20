# Py++ Usage and Examples

## Overview

This document provides example categories for writing and running Py++ programs. Each example file contains comments that explain the code.

## Example categories

- `examples/syntax/` — examples that demonstrate Py++ syntax.
- `examples/semantics/` — examples that highlight language semantics.
- `examples/macros/` — examples focused on macros and compile-time expansion.
- `examples/imports/` — examples for module imports.

## Running examples

To run a Py++ example file, use the CLI:

```bash
python -m pyplusplus.cli examples/syntax/hello_world.pypp --run
```

Or to dump the compiled Python AST:

```bash
python -m pyplusplus.cli examples/syntax/hello_world.pypp --dump-ast
```

## Example structure

Each example includes comments that explain the key language features and how the code works.

## `when` hooks examples

The `examples/semantics/classes_and_when.pypp` file demonstrates how `when` hooks are used inside a class to validate and control assignments to a field. This example shows practical semantics for guarded state updates in Py++.
