from __future__ import annotations

from pyplusplus.parser import parse_pyplusplus
from pyplusplus.ast_nodes import Assign, AugAssign, Await, Call, Constant, FromUse, Lambda, Match, MatchAs, MatchCase, MatchValue, Name, BinOp, Return, FunctionDef, BlockStmt, Subscript, TupleExpr, UnaryOp, ExprStmt

def test_parse_pyplusplus_simple_expression() -> None:
    source = "x = 1 + 2\n"
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    
    assert isinstance(node, Assign)
    assert isinstance(node.target, Name)
    assert isinstance(node.value, BinOp)
    assert isinstance(node.value.left, Constant) and isinstance(node.value.right, Constant)
    assert isinstance(node.value.left.value, int) and isinstance(node.value.right.value, int)
    
    assert node.target.id == "x"
    assert node.value.left.value == 1
    assert node.value.right.value == 2


def test_parse_pyplusplus_complex_expression() -> None:
    source = "result = (a + b) * c - d / e;\n"
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    
    assert isinstance(node, Assign)
    assert isinstance(node.target, Name)
    assert isinstance(node.value, BinOp)                    # (a + b) * c AND d / e
    assert isinstance(node.value.left, BinOp)               # (a + b) * c
    assert isinstance(node.value.left.left, BinOp)          # (a + b)
    assert isinstance(node.value.left.right, Name)          # c
    assert isinstance(node.value.left.left.left, Name)      # a
    assert isinstance(node.value.left.left.right, Name)     # b
    assert isinstance(node.value.right, BinOp)              # d / e
    assert isinstance(node.value.right.left, Name)          # d
    assert isinstance(node.value.right.right, Name)         # e
    
    # Check the variable names
    assert node.target.id == "result"
    assert node.value.left.left.left.id == "a"
    assert node.value.left.left.right.id == "b"
    assert node.value.left.right.id == "c"
    assert node.value.right.left.id == "d"
    assert node.value.right.right.id == "e"
    assert node.value.op == "-"
    assert node.value.left.op == "*"
    assert node.value.left.left.op == "+"
    assert node.value.right.op == "/"


def test_parse_pyplusplus_complex_expression_2() -> None:
    source = "~(5 & 3) | (8 ^ 12) << 2\n"
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    
    assert isinstance(node, ExprStmt)
    assert isinstance(node.value, BinOp)
    assert isinstance(node.value.left, UnaryOp)  # ~(5 & 3)
    assert isinstance(node.value.left.operand, BinOp)  # 5 & 3
    assert isinstance(node.value.left.operand.left, Constant)  # 5
    assert isinstance(node.value.left.operand.right, Constant)  # 3
    assert isinstance(node.value.right, BinOp)  # (8 ^ 12) << 2
    assert isinstance(node.value.right.left, BinOp)  # 8 ^ 12
    assert isinstance(node.value.right.left.left, Constant)  # 8
    assert isinstance(node.value.right.left.right, Constant)  # 12
    assert isinstance(node.value.right.right, Constant)  # 2
    
    assert node.value.op == "|"
    assert node.value.left.op == "~"
    assert node.value.left.operand.op == "&"
    assert node.value.right.op == "<<"
    assert node.value.right.left.op == "^"
    assert node.value.left.operand.left.value == 5
    assert node.value.left.operand.right.value == 3
    assert node.value.right.left.left.value == 8
    assert node.value.right.left.right.value == 12
    assert node.value.right.right.value == 2


def test_parse_pyplusplus_assignment_with_unary_operator() -> None:
    source = "x = -y;\n"
    tree = parse_pyplusplus(source)
    node = tree.body[0]

    assert isinstance(node, Assign)
    assert isinstance(node.target, Name)
    assert isinstance(node.value, UnaryOp)
    assert isinstance(node.value.operand, Name)

    assert node.target.id == "x"
    assert node.value.op == "-"
    assert node.value.operand.id == "y"


def test_parse_pyplusplus_unary_expression() -> None:
    source = "result = -5 + ~3;\n"
    tree = parse_pyplusplus(source)
    node = tree.body[0]

    assert isinstance(node, Assign)
    assert isinstance(node.target, Name)
    assert isinstance(node.value, BinOp)
    assert isinstance(node.value.left, UnaryOp)  # -5
    assert isinstance(node.value.left.operand, Constant)  # 5
    assert isinstance(node.value.right, UnaryOp)  # ~3
    assert isinstance(node.value.right.operand, Constant)  # 3

    assert node.target.id == "result"
    assert node.value.op == "+"
    assert node.value.left.op == "-"
    assert node.value.left.operand.value == 5
    assert node.value.right.op == "~"
    assert node.value.right.operand.value == 3


def test_parse_pyplusplus_augmented_assignments() -> None:
    source = (
        "z += 1;\n"
        "y -= 2;\n"
        "x *= 3;\n"
        "w /= 4;\n"
        "v %= 5;\n"
        "u **= 6;\n"
        "t &= 7;\n"
        "s |= 8;\n"
        "r ^= 9;\n"
        "q <<= 10;\n"
        "p >>= 11;\n"
    )
    tree = parse_pyplusplus(source)
    
    expected_ops = ["+=", "-=", "*=", "/=", "%=", "**=", "&=", "|=", "^=", "<<=", ">>="]
    for i, node in enumerate(tree.body):
        assert isinstance(node, AugAssign)
        assert isinstance(node.target, Name)
        assert isinstance(node.value, Constant)
        assert node.value.value == i + 1  # Check that the value is 1, 2, ..., 11
        assert node.op == expected_ops[i]  # The operator for AugAssign is stored in node.op
        assert node.target.id == chr(ord('z') - i)  # Check variable names from 'z' to 'p'


def test_parse_pyplusplus_function_definition() -> None:
    source = (
        'func add(a, b: int) -> int {\n'
        '    return a + b;\n'
        '}\n'
    )
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    
    assert isinstance(node, FunctionDef)
    assert isinstance(node.args[1].annotation, Name)
    assert isinstance(node.return_type, Name)
    assert isinstance(node.body, BlockStmt)
    assert isinstance(node.body.body[0], Return)
    assert isinstance(node.body.body[0].value, BinOp)
    assert isinstance(node.body.body[0].value.left, Name)
    assert isinstance(node.body.body[0].value.right, Name)
    
    assert node.decorators == []
    assert not node.is_async
    assert node.return_type is not None
    assert node.return_type.id == "int"
    assert node.name == "add"
    assert len(node.args) == 2
    assert node.args[0].name == "a"
    assert node.args[0].annotation is None
    assert node.args[1].name == "b"
    assert node.args[1].annotation.id == "int"
    assert len(node.body.body) == 1
    assert node.body.body[0].value.left.id == "a"
    assert node.body.body[0].value.right.id == "b"


def test_parse_pyplusplus_function_definition_with_no_return_type() -> None:
    source = (
        'func greet(name: str) {\n'
        '    print("Hello, " + name);\n'
        '}\n'
    )
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    
    assert isinstance(node, FunctionDef)
    assert isinstance(node.args[0].annotation, Name)
    assert node.return_type is None
    assert isinstance(node.body, BlockStmt)
    assert isinstance(node.body.body[0], ExprStmt)
    assert isinstance(node.body.body[0].value, Call)
    assert isinstance(node.body.body[0].value.func, Name)
    assert isinstance(node.body.body[0].value.args[0], BinOp)
    assert isinstance(node.body.body[0].value.args[0].left, Constant)
    assert isinstance(node.body.body[0].value.args[0].right, Name)
    
    assert node.decorators == []
    assert not node.is_async
    assert node.return_type is None
    assert node.name == "greet"
    assert len(node.args) == 1
    assert node.args[0].name == "name"
    assert node.args[0].annotation.id == "str"
    assert len(node.body.body) == 1
    assert node.body.body[0].value.func.id == "print"
    assert len(node.body.body[0].value.args) == 1
    assert node.body.body[0].value.args[0].left.value == "Hello, "
    assert node.body.body[0].value.args[0].right.id == "name"


def test_parse_pyplusplus_empty_function() -> None:
    source = (
        'func empty() {\n'
        '}\n'
    )
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    
    assert isinstance(node, FunctionDef)
    assert isinstance(node.body, BlockStmt)
    
    assert node.decorators == []
    assert not node.is_async
    assert node.return_type is None
    assert node.name == "empty"
    assert len(node.args) == 0
    assert len(node.body.body) == 0


def test_parse_pyplusplus_function_with_full_annotations() -> None:
    source = (
        'func compute(x: float, y: float) -> float {\n'
        '    return x * y;\n'
        '}\n'
    )
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    
    assert isinstance(node, FunctionDef)
    assert isinstance(node.args[0].annotation, Name)
    assert isinstance(node.args[1].annotation, Name)
    assert isinstance(node.return_type, Name)
    assert isinstance(node.body, BlockStmt)
    assert isinstance(node.body.body[0], Return)
    assert isinstance(node.body.body[0].value, BinOp)
    assert isinstance(node.body.body[0].value.left, Name)
    assert isinstance(node.body.body[0].value.right, Name)
    
    assert node.decorators == []
    assert not node.is_async
    assert node.return_type.id == "float"
    assert node.name == "compute"
    assert len(node.args) == 2
    assert node.args[0].name == "x"
    assert node.args[0].annotation.id == "float"
    assert node.args[1].name == "y"
    assert node.args[1].annotation.id == "float"
    assert len(node.body.body) == 1
    assert node.body.body[0].value.left.id == "x"
    assert node.body.body[0].value.right.id == "y"
    assert node.body.body[0].value.op == "*"


def test_parse_pyplusplus_functions_with_all_additional_features() -> None:
    source = (
        '@decorator1\n'
        '@decorator2(param)\n'
        'async func complex_func(a: int, b: str) -> bool {\n'
        '    result = await some_async_function(a, b);\n'
        '    return result;\n'
        '}\n'
    )
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    print(node.__repr__())
    
    assert isinstance(node, FunctionDef)
    assert isinstance(node.args[0].annotation, Name)
    assert isinstance(node.args[1].annotation, Name)
    assert isinstance(node.return_type, Name)
    assert isinstance(node.body, BlockStmt)
    assert isinstance(node.body.body[0], Assign)
    assert isinstance(node.body.body[0].target, Name)
    assert isinstance(node.body.body[0].value, Await)
    assert isinstance(node.body.body[0].value.value, Call)
    assert isinstance(node.body.body[0].value.value.func, Name)
    assert isinstance(node.body.body[0].value.value.args[0], Name)
    assert isinstance(node.body.body[0].value.value.args[1], Name)
    assert isinstance(node.body.body[1], Return)
    assert isinstance(node.body.body[1].value, Name)
    assert isinstance(node.decorators[0], Name)
    assert isinstance(node.decorators[1], Call)
    assert isinstance(node.decorators[1].func, Name)
    assert isinstance(node.decorators[1].args[0], Name)
    
    assert node.decorators[0].id == "decorator1"
    assert node.decorators[1].func.id == "decorator2"
    assert node.decorators[1].args[0].id == "param"
    assert node.is_async
    assert node.return_type.id == "bool"
    assert node.name == "complex_func"
    assert len(node.args) == 2
    assert node.args[0].name == "a"
    assert node.args[0].annotation.id == "int"
    assert node.args[1].name == "b"
    assert node.args[1].annotation.id == "str"
    assert len(node.body.body) == 2
    assert node.body.body[0].target.id == "result"
    assert node.body.body[0].value.value.func.id == "some_async_function"
    assert node.body.body[0].value.value.args[0].id == "a"
    assert node.body.body[0].value.value.args[1].id == "b"
    assert node.body.body[1].value.id == "result"


def test_parse_pyplusplus_function_returning_function() -> None:
    source = (
        'from typing use Callable as Call;\n'
        'func outer(x: int) -> Call[int, int] {\n'
        '    func inner(y: int) -> int {\n'
        '        return x + y;\n'
        '    };\n'
        '    return inner;\n'
        '}\n'
    )
    tree = parse_pyplusplus(source)
    use_node = tree.body[0]
    node = tree.body[1]
    
    print(node.__repr__())
    
    assert isinstance(use_node, FromUse)
    assert isinstance(use_node.source.path, Name)
    assert isinstance(node, FunctionDef)
    assert isinstance(node.args[0].annotation, Name)
    assert isinstance(node.return_type, Subscript)
    assert isinstance(node.return_type.value, Name)
    assert isinstance(node.return_type.slice, TupleExpr)
    assert isinstance(node.return_type.slice.elements[0], Name)
    assert isinstance(node.return_type.slice.elements[1], Name)
    assert isinstance(node.body, BlockStmt)
    assert isinstance(node.body.body[0], FunctionDef)
    assert isinstance(node.body.body[0].args[0].annotation, Name)
    assert isinstance(node.body.body[0].return_type, Name)
    assert isinstance(node.body.body[0].body, BlockStmt)
    assert isinstance(node.body.body[0].body.body[0], Return)
    assert isinstance(node.body.body[0].body.body[0].value, BinOp)
    assert isinstance(node.body.body[0].body.body[0].value.left, Name)
    assert isinstance(node.body.body[0].body.body[0].value.right, Name)
    
    assert use_node.source.path.id == "typing"
    assert use_node.names[0].name == "Callable"
    assert use_node.names[0].alias == "Call"
    assert node.decorators == []
    assert not node.is_async
    assert node.name == "outer"
    assert len(node.args) == 1
    assert node.args[0].name == "x"
    assert node.args[0].annotation.id == "int"
    assert node.return_type.value.id == "Call"
    assert len(node.return_type.slice.elements) == 2
    assert node.return_type.slice.elements[0].id == "int"
    assert node.return_type.slice.elements[1].id == "int"
    assert len(node.body.body) == 2
    assert node.body.body[0].name == "inner"
    assert len(node.body.body[0].args) == 1
    assert node.body.body[0].args[0].name == "y"
    assert node.body.body[0].args[0].annotation.id == "int"
    assert node.body.body[0].return_type.id == "int"
    assert len(node.body.body[0].body.body) == 1
    assert node.body.body[0].body.body[0].value.left.id == "x"
    assert node.body.body[0].body.body[0].value.right.id == "y"
    assert node.body.body[0].body.body[0].value.op == "+"


def test_parse_pyplusplus_lambda_expression() -> None:
    source = (
        '(func(x: int) -> int {\n'
        '    x * 2\n'
        '})(5);\n'
    )
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    print(node.__repr__())
    
    assert isinstance(node, ExprStmt)
    assert isinstance(node.value, Call)
    assert isinstance(node.value.func, Lambda)
    assert isinstance(node.value.args[0], Constant)
    assert isinstance(node.value.func.args[0].annotation, Name)
    assert isinstance(node.value.func.return_type, Name)
    assert isinstance(node.value.func.expr, BinOp)
    assert isinstance(node.value.func.expr.left, Name)
    assert isinstance(node.value.func.expr.right, Constant)
    
    assert node.value.args[0].value == 5
    assert node.value.func.args[0].name == "x"
    assert node.value.func.args[0].annotation.id == "int"
    assert node.value.func.return_type.id == "int"
    assert node.value.func.expr.left.id == "x"
    assert node.value.func.expr.right.value == 2


def test_parse_pyplusplus_lambda_in_lambda_expression() -> None:
    source = (
        '(func(x: int) -> int {\n'
        '    (func(y: int) -> int {\n'
        '        y * 2\n'
        '    })(x)\n'
        '})(1);\n'
    )
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    
    assert isinstance(node, ExprStmt)
    assert isinstance(node.value, Call)
    assert isinstance(node.value.func, Lambda)
    assert isinstance(node.value.args[0], Constant)
    assert isinstance(node.value.func.args[0].annotation, Name)
    assert isinstance(node.value.func.return_type, Name)
    assert isinstance(node.value.func.expr, Call)
    assert isinstance(node.value.func.expr.func, Lambda)
    assert isinstance(node.value.func.expr.func.args[0].annotation, Name)
    assert isinstance(node.value.func.expr.func.return_type, Name)
    assert isinstance(node.value.func.expr.func.expr, BinOp)
    assert isinstance(node.value.func.expr.func.expr.left, Name)
    assert isinstance(node.value.func.expr.func.expr.right, Constant)
    assert isinstance(node.value.func.expr.args[0], Name)
    
    assert node.value.args[0].value == 1
    assert node.value.func.args[0].name == "x"
    assert node.value.func.args[0].annotation.id == "int"
    assert node.value.func.return_type.id == "int"
    assert node.value.func.expr.func.args[0].name == "y"
    assert node.value.func.expr.func.args[0].annotation.id == "int"
    assert node.value.func.expr.func.return_type.id == "int"
    assert node.value.func.expr.func.expr.left.id == "y"
    assert node.value.func.expr.func.expr.right.value == 2
    assert node.value.func.expr.args[0].id == "x"
    assert node.value.func.expr.func.expr.op == "*"


def test_parse_pyplusplus_assigning_lambda_to_variable_with_annotation() -> None:
    source = (
        'double = func(x: int) -> int {\n'
        '    x * 2\n'
        '};\n'
    )
    tree = parse_pyplusplus(source)
    node = tree.body[0]
    
    assert isinstance(node, Assign)
    assert isinstance(node.target, Name)
    assert isinstance(node.value, Lambda)
    assert isinstance(node.value.args[0].annotation, Name)
    assert isinstance(node.value.return_type, Name)
    assert isinstance(node.value.expr, BinOp)
    assert isinstance(node.value.expr.left, Name)
    assert isinstance(node.value.expr.right, Constant)
    
    assert node.target.id == "double"
    assert node.value.args[0].name == "x"
    assert node.value.args[0].annotation.id == "int"
    assert node.value.return_type.id == "int"
    assert node.value.expr.left.id == "x"
    assert node.value.expr.right.value == 2


def test_parse_pyplusplus_match_statement_with_single_case_body() -> None:
    source = (
        'match (x) {\n'
        '    case 1: print("one");\n'
        '    case _: print("other");\n'
        '}\n'
    )
    tree = parse_pyplusplus(source)
    node = tree.body[0]

    assert isinstance(node, Match)
    assert isinstance(node.subject, Name)
    assert node.subject.id == "x"
    assert len(node.cases) == 2
    assert isinstance(node.cases[0], MatchCase)
    assert isinstance(node.cases[0].pattern, MatchValue)
    assert isinstance(node.cases[0].body, BlockStmt)
    assert isinstance(node.cases[0].body.body[0], ExprStmt)
    assert isinstance(node.cases[1].pattern, MatchAs)
    assert node.cases[1].pattern.name is None
