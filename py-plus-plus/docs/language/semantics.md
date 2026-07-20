# Py++ Language Semantics

## Expressions and evaluation

Py++ expressions are evaluated in a Python-like manner.

- Arithmetic operators: `+`, `-`, `*`, `/`, `//`, `%`, `**`
- Comparison operators: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical operators: `and`, `or`, `not`
- Boolean and `None` values work like Python.
- Formatted string literals support embedded expressions using `"` and curly braces.

Example:

```pypp
result = x * 2 + 5;
is_valid = x > 0 and y != None;
print(`"result = {result}"`);
```

## Function semantics

Functions evaluate their arguments before execution, and `return` exits the function early.

- `return` with no expression returns `None`.
- Functions may have annotations, but these are used for syntax compatibility rather than runtime enforcement.

Example:

```pypp
func greet(name: str) {
    return "Hello, " + name;
}
```

## Class semantics

Classes are runtime objects that support fields, methods, and inheritance.

- Class bodies can contain `func` definitions and `when` hooks.
- `extends` creates a subclass relationship.
- `when` hooks intercept assignment to a field and can run custom logic.

Example:

```pypp
class Counter {
    when value(new_val) {
        if (new_val < 0) {
            raise ValueError("Value cannot be negative");
        }
        value = new_val;
    }
}
```

## Lambda semantics

Py++ supports lambda-style anonymous functions using `func(...)` expressions.

- A `func` expression can be assigned to a variable.
- Nested `func` expressions can create closures.

Example:

```pypp
square = func(x) {
    return x * x;
};
print(square(6));
```

## Import semantics

Imports follow a module path or string path semantics.

- `use "module.pypp" as alias;` loads a file module by path.
- `use "module.py" as alias;` loads a Python file.
- `use "module.pyc" as alias;` loads a compiled Python module.
- `use module_name as alias;` uses a Python or Py++ module name.
- `from "module.pypp" use name;` imports specific names from a module.
- `from "module.py" use "name" as alias;` avoids conflicts when the imported name is also a Py++ keyword.

Imported names become available under the alias or directly in the current scope.

## Macro semantics

Macros in Py++ are expanded at compile time.

- Macro definitions do not generate runtime code directly.
- Macro calls use `$name[...]` and the macro body is inserted before runtime execution.

Macro arguments can be expressions, statements, code blocks, or identifiers depending on the declared parameter kind.

Example:

```pypp
macro repeat[code: Code] {
    code;
    code;
}

$repeat[{ print("twice"); }];
```

## Import semantics

Imports follow a module path or string path semantics.

- `use "module.pypp" as alias;` loads a file module by path.
- `use module_name as alias;` uses a Python or Py++ module name.
- `from "module.pypp" use name;` imports specific names from a module.

Imported names become available under the alias or directly in the current scope.

## Control flow semantics

- `if` statements evaluate conditions and execute matching branches.
- `elif` chains are evaluated in order until a condition is true.
- `while` loops repeat until the condition becomes false and may include an optional `else` block when the loop completes normally.
- `for` loops iterate over iterable values and may include an optional `else` block when no `break` occurs.
- `match` performs structural pattern matching with `case` branches.
- `try` / `except` / `else` / `finally` works like Python exception handling.

Example:

```pypp
for (item in items) {
    if (item == target) {
        break;
    }
} else {
    print("target not found");
}

try {
    risky_operation();
} except ValueError as err {
    print("error", err);
} else {
    print("success");
} finally {
    cleanup();
}
```
