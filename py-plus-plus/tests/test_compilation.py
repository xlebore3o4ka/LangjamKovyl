from __future__ import annotations

from pyplusplus.compiler import compile_pyplusplus_to_python_ast, execute_pyplusplus
from pyplusplus.parser import parse_pyplusplus


def test_compile_and_execute_annotated_function() -> None:
    source = (
        'func add(x: int, y: int) -> int {'
        '    return x + y;'
        '}'
    )
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert "add" in namespace
    assert callable(namespace["add"])
    assert namespace["add"](2, 3) == 5
    assert namespace["add"].__annotations__["x"] is int
    assert namespace["add"].__annotations__["y"] is int
    assert namespace["add"].__annotations__["return"] is int


def test_compile_and_execute_annotated_assignment() -> None:
    source = "x: int = 10;"
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert "x" in namespace
    assert namespace["x"] == 10


def test_compile_and_execute_generic_annotation() -> None:
    source = (
        'from typing use Callable as Call;'
        'func outer(x: int) -> Call[int, int] {'
        '    return x;'
        '}'
    )
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert "outer" in namespace
    assert callable(namespace["outer"])
    assert namespace["outer"](5) == 5
    assert "return" in namespace["outer"].__annotations__
    return_annotation = namespace["outer"].__annotations__["return"]
    assert hasattr(return_annotation, "__args__")
    assert return_annotation.__args__ == (int, int)


def test_compile_python_ast_directly() -> None:
    source = "func identity(value: str) -> str { return value; }"
    module_ast = parse_pyplusplus(source)
    python_ast = compile_pyplusplus_to_python_ast(module_ast)

    compiled_code = compile(python_ast, "<pyplusplus>", "exec")
    namespace: dict[str, object] = {"__name__": "__main__"}
    exec(compiled_code, namespace, namespace)

    assert "identity" in namespace
    assert namespace["identity"]("hello") == "hello"


def test_compile_and_execute_use_string_import() -> None:
    source = 'use "typing" as t; func identity(value: t.Any) -> t.Any { return value; }'
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert "t" in namespace
    assert namespace["t"].__name__ == "typing"
    assert namespace["identity"]("hello") == "hello"


def test_compile_and_execute_pypp_file_import(tmp_path) -> None:
    module_path = tmp_path / "greet_module.pypp"
    module_path.write_text('func greet() -> str { return "hello"; }', encoding="utf-8")

    source = f'use "{module_path.as_posix()}" as greet_mod; func run() -> str {{ return greet_mod.greet(); }}'
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert "greet_mod" in namespace
    assert namespace["run"]() == "hello"


def test_compile_and_execute_pypp_from_use_import(tmp_path) -> None:
    module_path = tmp_path / "greet_module.pypp"
    module_path.write_text('func greet() -> str { return "hello"; }', encoding="utf-8")

    source = f'from "{module_path.as_posix()}" use greet; func run() -> str {{ return greet(); }}'
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert namespace["run"]() == "hello"


def test_compile_and_execute_std_run_python_source() -> None:
    source = (
        'use "libs_pypp/std.pypp" as std;'
        'result = std.run_python_source(\n'
        '    "print(\\\"hello\\\")"'
        ');'
    )
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert "result" in namespace
    assert namespace["result"] == 0


def test_compile_and_execute_std_run_pypp_source() -> None:
    source = (
        'use "libs_pypp/std.pypp" as std;\n'
        'result = std.run_pypp_source(\n'
        '    "print(\\\"hello\\\");"\n'
        ');'
    )
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert "result" in namespace
    assert namespace["result"] == 0


def test_compile_and_execute_macro_definition_is_ignored() -> None:
    source = (
        'macro swap[a: Identifier, b: Identifier] {'
        '    temp = a;'
        '    a = b;'
        '    b = temp;'
        '}'
        'a = 1;'
        'b = 2;'
        '$swap[a, b];'
    )
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert namespace["a"] == 2
    assert namespace["b"] == 1
    assert "swap" not in namespace


def test_compile_and_execute_stmt_macro() -> None:
    source = (
        'macro do_twice[code: Stmt] {'
        '    code;'
        '    code;'
        '}'
        'a = 0;'
        '$do_twice[{ a = a + 1; }];'
    )
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert namespace["a"] == 2


def test_compile_and_execute_expr_macro() -> None:
    source = (
        'macro add_one[x: Expr] {'
        '    x + 1'
        '}'
        'a = $add_one[2];'
    )
    module_ast = parse_pyplusplus(source)
    namespace = execute_pyplusplus(module_ast, {})

    assert namespace["a"] == 3
