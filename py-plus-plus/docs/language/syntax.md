# Py++ Language Syntax

## Comments

Py++ supports two kinds of comments:

- Single-line comments start with `##` and continue to the end of the line.
- Block comments start with `#*` and end with `*#`.

Example:

```pypp
## This is a single-line comment
#* This is a
   multi-line comment *#
```

## Statements and blocks

Py++ uses braces and semicolons to define statement boundaries.

- A block is written with `{ ... }`.
- Statements may end with a semicolon `;`.
- Empty statements are allowed between semicolons.

Example:

```pypp
if (x > 0) {
    print("positive");
}
```

## Function definitions

Functions are defined with the `func` keyword.

Syntax:

```pypp
func function_name(arg1, arg2) {
    // body
}
```

Optional return annotations are supported after a colon:

```pypp
func add(x: int, y: int): int {
    return x + y;
}
```

## Classes and inheritance

Classes are defined with the `class` keyword.

Syntax:

```pypp
class MyClass {
    // body
}
```

Inheritance uses the `extends` keyword:

```pypp
class Child extends Parent {
    // body
}
```

## `when` hooks

Py++ supports `when` hooks to intercept field assignment inside a class or object.

Syntax:

```pypp
when fieldName(param) {
    // body runs when fieldName is assigned
}
```

## Imports

Py++ supports `use` and `from ... use` imports.

- `use "module.pypp" as alias;`
- `use "module.py" as alias;`
- `use "module.pyc" as alias;`
- `use module_name as alias;`
- `from "module.pypp" use name;`
- `from "module.py" use "name" as alias;`

Using string imports is useful when imported names might conflict with Py++ keywords or reserved identifiers.

Example:

```pypp
use "math.pypp" as math;
from "utils.py" use "class" as class_helper;
```

## Macros

Macros are defined with the `macro` keyword and invoked with `$name[...]`.

Syntax:

```pypp
macro my_macro[param: Expr] {
    param;
}

$my_macro[{ print("Hello"); }];
```

Macro parameters can be declared with kinds like `Expr`, `Stmt`, `Identifier`, and `Code`.

## F-strings

Py++ supports formatted string literals (f-strings) using Python-style interpolation.

Example:

```pypp
name = "Py++";
print(`"Hello, {name}!"`);
```

## Control flow

Py++ supports standard control flow statements:

- `if` / `elif` / `else`
- `while` / `else`
- `for` / `else`
- `match` / `case`
- `try` / `except` / `else` / `finally`
- `with`

Example:

```pypp
if (condition) {
    print("yes");
} elif (other_condition) {
    print("maybe");
} else {
    print("no");
}
```
