"""AST definitions for the Py++ language.

This module declares the node types used by the parser and compiler.
The AST is intentionally simple and resembles Python's own AST, with
explicit statement and expression classes for each language construct.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import List, NamedTuple, Union
import enum

from pyplusplus.lexer import TokenValue

AliasType = Union[str, None]
UseSource = NamedTuple("UseSource", [("path", Union[str, "Name", "Attribute"],), ("is_string", bool)])
FromUseName = NamedTuple("FromUseName", [("name", str), ("is_string", bool), ("alias", AliasType)])
AssignTarget = Union["Name", "Attribute", "Subscript", "TupleExpr", "ListExpr"]
ArgKind = enum.Enum("ArgKind", ["POSITIONAL", "VARARG", "KWARG", "KEYWORD_ONLY"])
DecoratorTypes = Union["Name", "Attribute", "Call"]
ForLoopTarget = Union["Name", "TupleExpr", "ListExpr"]
BodyStmt = Union["BlockStmt", "Stmt"]
MacroParamKind = enum.Enum("MacroParamKind", ["EXPRESSION", "STATEMENT", "IDENTIFIER", "CODE"])
MacroParam = NamedTuple("MacroParam", [("name", str), ("type", MacroParamKind | None)])

class Node:
    """Base node class for all AST elements."""
    pass


class Stmt(Node):
    """Base class for all statement nodes."""
    pass


class Expr(Node):
    """Base class for all expression nodes."""
    pass


@dataclass(frozen=True)
class Module(Node):
    """Root AST node containing a top-level List of statements."""
    body: List[Stmt]


@dataclass(frozen=True)
class ExprStmt(Stmt):
    """A statement whose value is an expression."""
    value: Expr


@dataclass(frozen=True)
class Assign(Stmt):
    """
    Assignment statement with a target and a value expression.
    Example: `x = 5;` or `y = function();`.
    """
    target: AssignTarget
    value: Expr


@dataclass(frozen=True)
class AnnAssign(Stmt):
    """
    Annotated assignment statement with a target, annotation, and optional value.
    Example: `x: int = 5;` or `y: str;`.
    """
    target: AssignTarget
    annotation: Expr
    value: Expr | None
    simple: bool


@dataclass(frozen=True)
class AugAssign(Stmt):
    """
    Augmented assignment statement with a target, operator, and value.
    Example: `x += 1;` or `y *= 2;`.
    """
    target: AssignTarget
    op: str
    value: Expr


@dataclass(frozen=True)
class Use(Stmt):
    """
    A `use` statement importing a module or string reference.
    Automatically resolves `.pypp` or `.py` extensions if not specified.
    Can also include an optional alias for the imported module.
    Example: `use "module.pypp" as pypp_module;` or `use "module.py" as py_module;` or `use "module";`.
    """
    source: UseSource
    alias: AliasType


@dataclass(frozen=True)
class FromUse(Stmt):
    """A `from ... use ...` import statement.
    Example: `from "module.pypp" use function;` or `from "module.py" use `func` as function;`.
    """
    source: UseSource
    names: List[FromUseName]


@dataclass(frozen=True)
class Attribute(Expr):
    """
    Attribute access expression which retrieves a named attribute from an object.
    Example: `object.attribute` or `module.function`.
    """
    value: Expr
    attr: str


@dataclass
class If(Stmt):
    """
    Conditional statement with optional chained `elif` and `else` bodies.
    Example: `if condition { ... }
              elif other_condition { ... }
              else { ... }`.
    """
    test: Expr
    body: BodyStmt
    orelse: If | Stmt | None = None


@dataclass(frozen=True)
class While(Stmt):
    """
    `while` loop statement.
    Example: `while (condition) { ... }`.
    """
    test: Expr
    body: BodyStmt
    orelse: BodyStmt | None


@dataclass(frozen=True)
class For(Stmt):
    """
    `for` loop statement over an iterable.
    Example: `for item in collection { ... }`.
    """
    target: ForLoopTarget
    iter: Expr
    body: BodyStmt


@dataclass(frozen=True)
class Arg(Node):
    """
    Function argument metadata for definitions and calls.
    Example: `func function(arg1, arg2: int = 5) { ... }`.
    """
    name: str
    default: Expr | None = None
    annotation: Expr | None = None
    kind: ArgKind = ArgKind.POSITIONAL


@dataclass()
class FunctionDef(Stmt):
    """
    Function definition with arguments, body, decorators, and return annotation.
    Example: `func function(arg1, arg2: int) -> str { ... }`.
    """
    name: str
    args: List[Arg]
    body: BodyStmt
    decorators: List[DecoratorTypes]
    is_async: bool = False
    return_type: Expr | None = None


@dataclass()
class ClassDef(Stmt):
    """
    Class definition with optional base classes and decorators.
    Example: `class MyClass extends BaseClass { ... }`.
    """
    name: str
    bases: List[Expr]
    body: BlockStmt
    decorators: List[DecoratorTypes]


@dataclass(frozen=True)
class MacroDef(Stmt):
    """
    Macro definition for compile-time code generation.
    Example: `macro my_macro[arg1: Expr, arg2: Identifier] { ... }`.
    """
    name: str
    params: List[MacroParam]
    body: BlockStmt


@dataclass(frozen=True)
class MacroCall(Expr):
    """
    A call to a macro by name with argument expressions.
    Example: `$my_macro[arg1, arg2]`.
    """
    name: str
    args: List[Expr]


@dataclass(frozen=True)
class BlockExpr(Expr):
    """
    A block expression passed as a macro argument.
    Example: `$my_macro[{ ... }]`.
    """
    body: List[Stmt]


@dataclass(frozen=True)
class BlockStmt(Stmt):
    """
    A block of statements treated as a statement.
    Example: `{ ... }` or `let x = { ... };`.
    """
    body: List[Stmt]


@dataclass(frozen=True)
class When(Stmt):
    """
    A method-like hook attached to a class field or property.
    Will be invoked when the field is modified.
    Example: `when field_name(input) { ... }`.
    """
    var: str
    param: str
    body: BodyStmt


@dataclass(frozen=True)
class Return(Stmt):
    """
    Return statement, optionally returning a value.
    Example: `return;` or `return value;`.
    """
    value: Expr | None


@dataclass(frozen=True)
class Pass(Stmt):
    """
    No-op statement.
    Example: `pass;`.
    """
    pass


@dataclass(frozen=True)
class Name(Expr):
    """
    Identifier expression.
    Example: `variable_name` or `function_name`.
    """
    id: str


@dataclass(frozen=True)
class Constant(Expr):
    """
    Literal constant expression.
    Example: `42`, `3.14`, `"string"`, `True`, `None`.
    """
    value: TokenValue


@dataclass(frozen=True)
class FString(Expr):
    """
    Formatted string literal expression.
    Example: `f"Hello, {name}!"`, `f"{value:.2f}"`, `f"{expr:_<5}"`.
    """
    value: str


@dataclass(frozen=True)
class BinOp(Expr):
    """
    Binary operation expression with a left value, operator, and right value.
    Example: `x + y`, `a * b`, `value // 2`.
    """
    left: Expr
    op: str
    right: Expr


@dataclass(frozen=True)
class UnaryOp(Expr):
    """
    Unary operation expression like `-x` or `not x`.
    Example: `-a`, `not b`, `--c`, `d++`.
    """
    op: str
    operand: Expr


@dataclass(frozen=True)
class BoolOp(Expr):
    """
    Boolean operation expression combining multiple values.
    Example: `x and y`, `a or b`, `not c`.
    """
    op: str
    values: List[Expr]


@dataclass(frozen=True)
class Compare(Expr):
    """
    Comparison expression with one left value and many comparators.
    Example: `x < y`, `a == b`, `value >= 10`."""
    left: Expr
    ops: List[str]
    comparators: List[Expr]


@dataclass(frozen=True)
class ListExpr(Expr):
    """
    List literal expression.
    Example: `[1, 2, 3]`.
    """
    elements: List[Expr]


@dataclass(frozen=True)
class TupleExpr(Expr):
    """
    Tuple literal expression.
    Example: `(1, 2, 3)`.
    """
    elements: List[Expr]


@dataclass(frozen=True)
class DictExpr(Expr):
    """
    Dictionary literal expression.
    Example: `{"key": "value"}`.
    """
    keys: List[Expr]
    values: List[Expr]


@dataclass(frozen=True)
class SetExpr(Expr):
    """
    Set literal expression.
    Example: `{1, 2, 3}`.
    """
    elements: List[Expr]


@dataclass(frozen=True)
class Subscript(Expr):
    """
    Subscript expression for indexing or slicing.
    Example: `array[0]`, `matrix[i][j]`.
    """
    value: Expr
    slice: Expr


@dataclass(frozen=True)
class Slice(Expr):
    """
    Slice expression used inside subscripts.
    Example: `array[1:10:2]`.
    """
    lower: Expr | None
    upper: Expr | None
    step: Expr | None


@dataclass(frozen=True)
class IfExpr(Expr):
    """
    Inline conditional expression.
    Example: `x if condition else y`.
    """
    test: Expr
    body: Expr
    orelse: Expr


@dataclass(frozen=True)
class Lambda(Expr):
    """
    Anonymous function expression.
    Example: `func(x) { x + 1 }` or `a = func(x) { x + 1 };`.
    """
    args: List[Arg]
    return_type: Expr | None
    expr: Expr


@dataclass(frozen=True)
class Yield(Expr):
    """
    Yield expression used in generator functions.
    Example: `yield value;`.
    """
    value: Expr | None


@dataclass(frozen=True)
class ListComp(Expr):
    """
    List comprehension expression.
    Example: `[x for x in iterable]`.
    """
    elt: Expr
    generators: List[Comprehension]


@dataclass(frozen=True)
class SetComp(Expr):
    """
    Set comprehension expression.
    Example: `{x for x in iterable}`.
    """
    elt: Expr
    generators: List[Comprehension]


@dataclass(frozen=True)
class DictComp(Expr):
    """
    Dictionary comprehension expression.
    Example: `{k: v for k, v in iterable}`.
    """
    key: Expr
    value: Expr
    generators: List[Comprehension]


@dataclass(frozen=True)
class GeneratorExp(Expr):
    """
    Generator expression.
    Example: `(x for x in iterable)`.
    """
    elt: Expr
    generators: List[Comprehension]


@dataclass(frozen=True)
class Comprehension(Node):
    """
    Generator/comprehension clause with target, iterator, and optional conditions.
    Example: `for x in iterable if condition`.
    """
    target: AssignTarget
    iter: Expr
    ifs: List[Expr]


@dataclass(frozen=True)
class With(Stmt):
    """
    Context manager statement with optional aliasing.
    Example: `with (open("file.txt") as f) { ... }`.
    """
    context_expr: Expr
    alias: AliasType
    body: BodyStmt


@dataclass(frozen=True)
class Break(Stmt):
    """
    Loop break statement.
    Example: `break;`.
    """
    pass


@dataclass(frozen=True)
class Continue(Stmt):
    """
    Loop continue statement.
    Example: `continue;`.
    """
    pass


@dataclass(frozen=True)
class Match(Stmt):
    """
    Structural match statement with a subject and cases.
    Example: `match value { case 1: ... case 2: ... }`.
    """
    subject: Expr
    cases: List[MatchCase]


@dataclass(frozen=True)
class MatchCase(Node):
    """
    Single match case with pattern, optional guard, and body.
    Example: `case pattern if guard: ...`.
    """
    pattern: Pattern
    guard: Expr | None
    body: BodyStmt


class Pattern(Node):
    """
    Base class for match patterns.
    Example: `case pattern: ...`.
    """
    pass


@dataclass(frozen=True)
class MatchValue(Pattern):
    """
    Match pattern that compares against a value.
    Example: `case 42: ...` or `case "string": ...`.
    """
    value: Expr


@dataclass(frozen=True)
class MatchAs(Pattern):
    """
    Match pattern that binds a name or wildcard `_`.
    Example: `case x: ...` or `case _: ...`.
    """
    name: str | None


@dataclass(frozen=True)
class MatchSequence(Pattern):
    """
    Match pattern for sequence destructuring.
    Example: `case [x, y, z]: ...`.
    """
    patterns: List[Pattern]


@dataclass(frozen=True)
class MatchOr(Pattern):
    """
    Match pattern representing alternatives.
    Example: `case 1 | 2 | 3: ...`.
    """
    patterns: List[Pattern]


@dataclass(frozen=True)
class Call(Expr):
    """
    Function or method call expression.
    Example: `func(arg1, arg2)` or `obj.method(arg)`.
    """
    func: Expr
    args: List[Expr]
    keywords: List[tuple[str, Expr]]
    starargs: List[Expr]
    kwargs: List[Expr]


@dataclass(frozen=True)
class Assert(Stmt):
    """
    Assert statement to validate conditions at runtime.
    Example: `assert condition, "message";`.
    """
    test: Expr
    msg: Expr | None


@dataclass(frozen=True)
class Raise(Stmt):
    """
    Raise exception statement.
    Example: `raise Exception("error");`.
    """
    exc: Expr | None


@dataclass(frozen=True)
class ExceptHandler(Node):
    """
    Exception handler used inside try/except blocks.
    Example: `except (Exception as e) { ... }`.
    """
    type: Expr | None
    name: str | None
    body: BodyStmt


@dataclass(frozen=True)
class Try(Stmt):
    """
    Try statement with handlers, else block, and finally block.
    Example: `try { ... } except (Exception as e) { ... } else { ... } finally { ... }`.
    """
    body: BodyStmt
    handlers: List[ExceptHandler]
    orelse: BodyStmt | None
    finalbody: BodyStmt | None


@dataclass(frozen=True)
class Del(Stmt):
    """
    Delete statement removing one or more targets.
    Example: `del x, y, z;`.
    """
    targets: List[AssignTarget]


@dataclass(frozen=True)
class Global(Stmt):
    """
    Declare names as global within a function.
    Example: `global x, y, z;`.
    """
    names: List[str]


@dataclass(frozen=True)
class Nonlocal(Stmt):
    """
    Declare names as nonlocal within nested functions.
    Example: `nonlocal x, y, z;`.
    """
    names: List[str]


@dataclass(frozen=True)
class Await(Expr):
    """
    Await expression for async operations.
    Example: `await coro();`.
    """
    value: Expr
