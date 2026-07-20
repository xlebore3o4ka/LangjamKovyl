from __future__ import annotations

import ast
import copy
import textwrap
from pathlib import Path
from typing import Callable, Dict, Sequence

from .parser import parse_pyplusplus_file
from .ast_nodes import (
    ArgKind,
    AnnAssign,
    Assert,
    Assign,
    AssignTarget,
    AugAssign,
    Attribute,
    Await,
    BinOp,
    BodyStmt,
    BoolOp,
    BlockExpr,
    BlockStmt,
    Call,
    ClassDef,
    Compare,
    Constant,
    Comprehension,
    Continue,
    Del,
    DictComp,
    DictExpr,
    Expr,
    ExprStmt,
    For,
    ForLoopTarget,
    FromUse,
    FunctionDef,
    GeneratorExp,
    Global,
    If,
    IfExpr,
    Lambda,
    List,
    ListComp,
    MacroCall,
    MacroDef,
    MacroParam,
    MacroParamKind,
    Match,
    MatchAs,
    MatchOr,
    MatchSequence,
    MatchValue,
    Module,
    Name,
    FString,
    ListExpr,
    Node,
    Nonlocal,
    Pass,
    Pattern,
    Raise,
    Return,
    Slice,
    Stmt,
    Subscript,
    SetComp,
    SetExpr,
    Try,
    TupleExpr,
    UnaryOp,
    Use,
    UseSource,
    When,
    With,
    While,
    Yield,
    Break,
)


def compile_pyplusplus_to_python_ast(source_ast: Module, source_path: str | Path | None = None) -> ast.Module:
    """
    Compile a Py++ AST into a Python AST ready for execution.
    
    Arguments:
        source_ast: The Py++ AST to compile.
        source_path: Optional path to the source file, used for resolving relative imports.
    Returns:
        A Python AST module object.
    """
    use_hooks = _has_when(source_ast)
    use_imports = _has_use(source_ast)
    body: List[ast.stmt] = []
    if use_hooks:
        body.extend([_compile_hooks_dict(), _compile_apply_hook_function()])
    if use_imports:
        body.extend(_compile_use_helpers())
    base_path = Path(source_path).resolve().parent if source_path is not None else Path.cwd()
    macros = _collect_macros(source_ast, base_path)
    for stmt in source_ast.body:
        body.extend(_compile_stmt_list(stmt, use_hooks, macros=macros, source_base_path=base_path))
    module = ast.Module(body=body, type_ignores=[])
    ast.fix_missing_locations(module)
    return module


def execute_pyplusplus(source_ast: Module, globals_dict: Dict[str, object] | None = None, source_file: str | None = None) -> Dict[str, object]:
    """Compile and execute Py++ code from its AST representation."""
    compiled_ast = compile_pyplusplus_to_python_ast(copy.deepcopy(source_ast), source_path=source_file)
    code_obj = compile(compiled_ast, "<pyplusplus>", "exec", dont_inherit=True)
    if globals_dict is None:
        globals_dict = {"__name__": "__main__"}
    else:
        globals_dict.setdefault("__name__", "__main__")
    if source_file is not None:
        globals_dict["__file__"] = source_file
    else:
        globals_dict.setdefault("__file__", "<pyplusplus>")
    exec(code_obj, globals_dict, globals_dict)
    return globals_dict


def _iter_stmt_body(body: Stmt | List[Stmt] | BlockStmt | None) -> List[Stmt]:
    if body is None:
        return []
    if isinstance(body, list):
        return body
    if isinstance(body, BlockStmt):
        return body.body
    return [body]


def _has_when(node: Module | Stmt) -> bool:
    if isinstance(node, Module):
        return any(_has_when(stmt) for stmt in node.body)
    if isinstance(node, When):
        return True
    if isinstance(node, (If, While, For, FunctionDef, ClassDef)):
        body = _iter_stmt_body(getattr(node, "body", None))
        orelse = _iter_stmt_body(getattr(node, "orelse", None))
        return any(_has_when(stmt) for stmt in body + orelse)
    return False


def _has_use(node: Module | Stmt) -> bool:
    if isinstance(node, Module):
        return any(_has_use(stmt) for stmt in node.body)
    if isinstance(node, Use) or isinstance(node, FromUse):
        return True
    if isinstance(node, (If, While, For, FunctionDef)):
        body = _iter_stmt_body(getattr(node, "body", None))
        orelse = _iter_stmt_body(getattr(node, "orelse", None))
        return any(_has_use(stmt) for stmt in body + orelse)
    return False


def _use_source_to_string(source: UseSource) -> str:
    if isinstance(source.path, str):
        return source.path
    return _expr_to_source(source.path, {})


def _collect_macros(source_ast: Module, base_path: Path) -> Dict[str, MacroDef]:
    macros: Dict[str, MacroDef] = {}
    for statement in source_ast.body:
        if isinstance(statement, MacroDef):
            macros[statement.name] = statement
        elif isinstance(statement, Use):
            path_str = _use_source_to_string(statement.source)
            if path_str.endswith(".pypp"):
                imported_path = Path(path_str)
                if not imported_path.is_absolute():
                    imported_path = base_path / imported_path
                if imported_path.exists():
                    imported_ast = parse_pyplusplus_file(imported_path)
                    imported_macros = _collect_macros(imported_ast, imported_path.resolve().parent)
                    macros.update(imported_macros)
        elif isinstance(statement, FromUse):
            path_str = _use_source_to_string(statement.source)
            imported_path = Path(path_str)
            if not imported_path.is_absolute():
                imported_path = base_path / imported_path
            if imported_path.suffix == ".pypp" and imported_path.exists():
                imported_ast = parse_pyplusplus_file(imported_path)
                imported_macros = _collect_macros(imported_ast, imported_path.resolve().parent)
                macros.update(imported_macros)
                for name, _, alias in statement.names:
                    if name in imported_macros:
                        macros[alias or name] = imported_macros[name]
            else:
                imported_pypp = imported_path.with_suffix(".pypp")
                if imported_pypp.exists():
                    imported_ast = parse_pyplusplus_file(imported_pypp)
                    imported_macros = _collect_macros(imported_ast, imported_pypp.resolve().parent)
                    macros.update(imported_macros)
                    for name, _, alias in statement.names:
                        if name in imported_macros:
                            macros[alias or name] = imported_macros[name]
    return macros


def _compile_macro_call(node: MacroCall, macros: Dict[str, MacroDef], use_hooks: bool) -> List[ast.stmt]:
    if node.name not in macros:
        raise NameError(f"Macro not defined: {node.name}")
    macro = macros[node.name]
    if len(node.args) != len(macro.params):
        raise SyntaxError(f"Macro {node.name} expects {len(macro.params)} arguments, got {len(node.args)}")
    _validate_macro_args(macro, node.args)
    arg_map = {param.name: arg for param, arg in zip(macro.params, node.args)}
    renamed_body = _apply_macro_hygiene(copy.deepcopy(macro.body.body), macro.params)
    expanded: List[Stmt] = []
    for stmt in renamed_body:
        substituted = _substitute_macro_statement(stmt, arg_map, macros)
        if isinstance(substituted, list):
            expanded.extend(substituted)
        else:
            expanded.append(substituted)
    compiled: List[ast.stmt] = []
    for stmt in expanded:
        compiled.extend(_compile_stmt_list(stmt, use_hooks, macros=macros))
    return compiled


def _macro_body_is_expr(macro: MacroDef) -> bool:
    if len(macro.body.body) != 1:
        return False
    return isinstance(macro.body.body[0], (ExprStmt, Return))


def _compile_macro_expr(node: MacroCall, macros: Dict[str, MacroDef]) -> ast.expr:
    if node.name not in macros:
        raise NameError(f"Macro not defined: {node.name}")
    macro = macros[node.name]
    if not _macro_body_is_expr(macro):
        raise SyntaxError(f"Macro {node.name} cannot be used in expression context")
    if len(node.args) != len(macro.params):
        raise SyntaxError(f"Macro {node.name} expects {len(macro.params)} arguments, got {len(node.args)}")
    _validate_macro_args(macro, node.args)
    arg_map = {param.name: arg for param, arg in zip(macro.params, node.args)}
    renamed_body = _apply_macro_hygiene(copy.deepcopy(macro.body.body), macro.params)
    substituted = _substitute_macro_statement(renamed_body[0], arg_map)
    if isinstance(substituted, list):
        raise SyntaxError(f"Macro {node.name} returned block statements in expression context")
    if isinstance(substituted, ExprStmt):
        return _compile_expr(substituted.value, macros)
    if isinstance(substituted, Return):
        if substituted.value is None:
            return ast.Constant(value=None)
        return _compile_expr(substituted.value, macros)
    raise TypeError(f"Unsupported macro expression body: {type(substituted).__name__}")


def _apply_macro_hygiene(body: List[Stmt], params: List[MacroParam]) -> List[Stmt]:
    param_names = {param.name for param in params}
    local_names = _collect_macro_local_names(body)
    rename_map = {name: f"{name}__pypp_{id(body)}" for name in local_names if name not in param_names}
    result: List[Stmt] = []
    for stmt in body:
        renamed_stmt = _rename_macro_names(stmt, rename_map)
        if isinstance(renamed_stmt, list):
            result.extend(renamed_stmt)
        elif isinstance(renamed_stmt, Stmt):
            result.append(renamed_stmt)
        else:
            raise TypeError(f"Expected Stmt or List[Stmt], got {type(renamed_stmt).__name__}")
    return result


def _collect_macro_local_names(body: List[Stmt]) -> set[str]:
    names: set[str] = set()
    for stmt in body:
        names.update(_collect_names_from_statement(stmt))
    return names


def _collect_names_from_statement(node: Stmt) -> set[str]:
    collected: set[str] = set()
    if isinstance(node, Assign):
        collected.update(_collect_names_from_target(node.target))
    elif isinstance(node, AugAssign):
        collected.update(_collect_names_from_target(node.target))
    elif isinstance(node, For):
        collected.update(_collect_names_from_target(node.target))
        if isinstance(node.body, BlockStmt):
            for stmt in node.body.body:
                collected.update(_collect_names_from_statement(stmt))
        else:
            collected.update(_collect_names_from_statement(node.body))
    elif isinstance(node, With):
        if isinstance(node.body, BlockStmt):
            for stmt in node.body.body:
                collected.update(_collect_names_from_statement(stmt))
        else:
            collected.update(_collect_names_from_statement(node.body))
    elif isinstance(node, If):
        if isinstance(node.body, BlockStmt):
            for stmt in node.body.body:
                collected.update(_collect_names_from_statement(stmt))
        else:
            collected.update(_collect_names_from_statement(node.body))
        if isinstance(node.orelse, BlockStmt):
            for stmt in node.orelse.body:
                collected.update(_collect_names_from_statement(stmt))
        elif isinstance(node.orelse, If):
            collected.update(_collect_names_from_statement(node.orelse))
        elif isinstance(node.orelse, Stmt):
            collected.update(_collect_names_from_statement(node.orelse))
    elif isinstance(node, FunctionDef):
        if isinstance(node.body, BlockStmt):
            for stmt in node.body.body:
                collected.update(_collect_names_from_statement(stmt))
        else:
            collected.update(_collect_names_from_statement(node.body))
    elif isinstance(node, ExprStmt) and isinstance(node.value, Name):
        pass
    return collected


def _collect_names_from_target(node: Expr | Stmt) -> set[str]:
    if isinstance(node, Name):
        return {node.id}
    if isinstance(node, Attribute):
        return set()
    if isinstance(node, Subscript):
        return _collect_names_from_target(node.value)
    if isinstance(node, TupleExpr) or isinstance(node, ListExpr):
        collected: set[str] = set()
        for elem in node.elements:
            collected.update(_collect_names_from_target(elem))
        return collected
    return set()


def _rename_macro_names(node: Node, rename_map: Dict[str, str]) -> Node:
    if isinstance(node, Name):
        return Name(id=rename_map.get(node.id, node.id))
    if isinstance(node, Assign):
        target = _rename_macro_names(node.target, rename_map)
        value = _rename_macro_names(node.value, rename_map)
        
        if not _is_assign_target(target):
            raise TypeError(f"Expected AssignTarget, got {type(target).__name__}")
        if not isinstance(value, Expr):
            raise TypeError(f"Expected Expr, got {type(value).__name__}")
        return Assign(
            target=target,
            value=value,
        )
    if isinstance(node, AugAssign):
        target = _rename_macro_names(node.target, rename_map)
        value = _rename_macro_names(node.value, rename_map)
        if not _is_assign_target(target):
            raise TypeError(f"Expected AssignTarget, got {type(target).__name__}")
        if not isinstance(value, Expr):
            raise TypeError(f"Expected Expr, got {type(value).__name__}")
        
        return AugAssign(
            target=target,
            op=node.op,
            value=value,
        )
    if isinstance(node, ExprStmt):
        value = _rename_macro_names(node.value, rename_map)
        if not isinstance(value, Expr):
            raise TypeError(f"Expected Expr, got {type(value).__name__}")
        return ExprStmt(value=value)
    if isinstance(node, If):
        test = _rename_macro_names(node.test, rename_map)
        statements: List[Stmt] = []
        for stmt in _body_to_list(node.body):
            renamed_stmt = _rename_macro_names(stmt, rename_map)
            if isinstance(renamed_stmt, list):
                statements.extend(renamed_stmt)
            elif isinstance(renamed_stmt, Stmt):
                statements.append(renamed_stmt)
            else:
                raise TypeError(f"Expected Stmt or List[Stmt], got {type(renamed_stmt).__name__}")
        body_stmt = _build_body_stmt(
            node.body,
            statements,
        )
        if isinstance(node.orelse, If):
            orelse = _rename_macro_names(node.orelse, rename_map)
        else:
            statements: List[Stmt] = []
            for stmt in _body_to_list(node.orelse):
                renamed_stmt = _rename_macro_names(stmt, rename_map)
                if isinstance(renamed_stmt, list):
                    statements.extend(renamed_stmt)
                elif isinstance(renamed_stmt, Stmt):
                    statements.append(renamed_stmt)
                else:
                    raise TypeError(f"Expected Stmt or List[Stmt], got {type(renamed_stmt).__name__}")
            orelse = _build_optional_body_stmt(
                node.orelse,
                statements,
            )

        if not isinstance(test, Expr):
            raise TypeError(f"Expected Expr for test, got {type(test).__name__}")
        if orelse is not None and not _is_body_stmt(orelse):
            raise TypeError(f"Expected BodyStmt for orelse, got {type(orelse).__name__}")
        return If(
            test=test,
            body=body_stmt,
            orelse=orelse,
        )
    if isinstance(node, While):
        test = _rename_macro_names(node.test, rename_map)
        statements: List[Stmt] = []
        for stmt in _body_to_list(node.body):
            renamed_stmt = _rename_macro_names(stmt, rename_map)
            if isinstance(renamed_stmt, list):
                statements.extend(renamed_stmt)
            elif isinstance(renamed_stmt, Stmt):
                statements.append(renamed_stmt)
            else:
                raise TypeError(f"Expected Stmt or List[Stmt], got {type(renamed_stmt).__name__}")
        body_stmt = _build_body_stmt(
            node.body,
            statements,
        )
        statements: List[Stmt] = []
        for stmt in _body_to_list(node.orelse):
            renamed_stmt = _rename_macro_names(stmt, rename_map)
            if isinstance(renamed_stmt, list):
                statements.extend(renamed_stmt)
            elif isinstance(renamed_stmt, Stmt):
                statements.append(renamed_stmt)
            else:
                raise TypeError(f"Expected Stmt or List[Stmt], got {type(renamed_stmt).__name__}")
        orelse = _build_optional_body_stmt(
            node.orelse,
            statements,
        ) if node.orelse is not None else None
        
        if not isinstance(test, Expr):
            raise TypeError(f"Expected Expr for test, got {type(test).__name__}")
        return While(
            test=test,
            body=body_stmt,
            orelse=orelse,
        )
    if isinstance(node, For):
        target = node.target
        iter = _rename_macro_names(node.iter, rename_map)
        
        statements: List[Stmt] = []
        for stmt in _body_to_list(node.body):
            renamed_stmt = _rename_macro_names(stmt, rename_map)
            if isinstance(renamed_stmt, list):
                statements.extend(renamed_stmt)
            elif isinstance(renamed_stmt, Stmt):
                statements.append(renamed_stmt)
            else:
                raise TypeError(f"Expected Stmt or List[Stmt], got {type(renamed_stmt).__name__}")
        body_stmt = _build_body_stmt(
            node.body,
            statements,
        )
        
        target = _rename_macro_names(target, rename_map)
        if not isinstance(target, ForLoopTarget):
            raise TypeError(f"Expected ForLoopTarget for target, got {type(target).__name__}")
        if not isinstance(iter, Expr):
            raise TypeError(f"Expected Expr for iter, got {type(iter).__name__}")
        return For(
            target=target,
            iter=iter,
            body=body_stmt,
        )
    if isinstance(node, With):
        alias = node.alias
        if alias in rename_map:
            alias = rename_map[alias]
        
        context_expr = _rename_macro_names(node.context_expr, rename_map)  
        statements: List[Stmt] = []
        for stmt in _body_to_list(node.body):
            renamed_stmt = _rename_macro_names(stmt, rename_map)
            if isinstance(renamed_stmt, list):
                statements.extend(renamed_stmt)
            elif isinstance(renamed_stmt, Stmt):
                statements.append(renamed_stmt)
            else:
                raise TypeError(f"Expected Stmt or List[Stmt], got {type(renamed_stmt).__name__}")
        body_stmt = _build_body_stmt(
            node.body,
            statements,
        )
        
        if not isinstance(context_expr, Expr):
            raise TypeError(f"Expected Expr for context_expr, got {type(context_expr).__name__}")
        return With(
            context_expr=context_expr,
            alias=alias,
            body=body_stmt,
        )
    if isinstance(node, FunctionDef):
        statements: List[Stmt] = []
        for stmt in _body_to_list(node.body):
            renamed_stmt = _rename_macro_names(stmt, rename_map)
            if isinstance(renamed_stmt, list):
                statements.extend(renamed_stmt)
            elif isinstance(renamed_stmt, Stmt):
                statements.append(renamed_stmt)
            else:
                raise TypeError(f"Expected Stmt or List[Stmt], got {type(renamed_stmt).__name__}")
        body_stmt = _build_body_stmt(
            node.body,
            statements,
        )
        return FunctionDef(
            name=node.name,
            args=node.args,
            body=body_stmt,
            decorators=node.decorators,
            is_async=node.is_async,
            return_type=node.return_type,
        )
    if isinstance(node, Call):
        func = _rename_macro_names(node.func, rename_map)
        args: List[Expr] = []
        for arg in node.args:
            renamed_arg = _rename_macro_names(arg, rename_map)
            if not isinstance(renamed_arg, Expr):
                raise TypeError(f"Expected Expr for argument, got {type(renamed_arg).__name__}")
            args.append(renamed_arg)
        if not isinstance(func, Expr):
            raise TypeError(f"Expected Expr for func, got {type(func).__name__}")
        return Call(
            func=func,
            args=args,
            keywords=node.keywords,
            starargs=node.starargs,
            kwargs=node.kwargs,
        )
    if isinstance(node, Attribute):
        value = _rename_macro_names(node.value, rename_map)
        if not isinstance(value, Expr):
            raise TypeError(f"Expected Expr for value, got {type(value).__name__}")
        return Attribute(
            value=value,
            attr=node.attr
            )
    if isinstance(node, Subscript):
        value = _rename_macro_names(node.value, rename_map)
        slice = _rename_macro_names(node.slice, rename_map)
        if not isinstance(value, Expr):
            raise TypeError(f"Expected Expr for value, got {type(value).__name__}")
        if not isinstance(slice, Expr):
            raise TypeError(f"Expected Expr for slice, got {type(slice).__name__}")
        return Subscript(
            value=value,
            slice=slice
        )
    if isinstance(node, BinOp):
        left = _rename_macro_names(node.left, rename_map)
        right = _rename_macro_names(node.right, rename_map)
        
        if not isinstance(left, Expr):
            raise TypeError(f"Expected Expr for left operand, got {type(left).__name__}")
        if not isinstance(right, Expr):
            raise TypeError(f"Expected Expr for right operand, got {type(right).__name__}")
        return BinOp(
            left=left,
            op=node.op,
            right=right
            )
    if isinstance(node, UnaryOp):
        operand = _rename_macro_names(node.operand, rename_map)
        if not isinstance(operand, Expr):
            raise TypeError(f"Expected Expr for operand, got {type(operand).__name__}")
        return UnaryOp(
            op=node.op,
            operand=operand,
        )
    if isinstance(node, BoolOp):
        values: List[Expr] = []
        for value in node.values:
            renamed_value = _rename_macro_names(value, rename_map)
            if not isinstance(renamed_value, Expr):
                raise TypeError(f"Expected Expr for value in BoolOp, got {type(renamed_value).__name__}")
            values.append(renamed_value)
        
        return BoolOp(
            op=node.op,
            values=values
            )
    if isinstance(node, Compare):
        left = _rename_macro_names(node.left, rename_map)
        comparators: List[Expr] = []
        for comp in node.comparators:
            renamed_comp = _rename_macro_names(comp, rename_map)
            if not isinstance(renamed_comp, Expr):
                raise TypeError(f"Expected Expr for comparator in Compare, got {type(renamed_comp).__name__}")
            comparators.append(renamed_comp)
            
        if not isinstance(left, Expr):
            raise TypeError(f"Expected Expr for left operand in Compare, got {type(left).__name__}")
        return Compare(
            left=left,
            ops=node.ops,
            comparators=comparators
        )
    if isinstance(node, ListExpr):
        elements: List[Expr] = []
        for elem in node.elements:
            renamed_elem = _rename_macro_names(elem, rename_map)
            if not isinstance(renamed_elem, Expr):
                raise TypeError(f"Expected Expr for element in ListExpr, got {type(renamed_elem).__name__}")
            elements.append(renamed_elem)
        return ListExpr(
            elements=elements
        )
    if isinstance(node, TupleExpr):
        elements: List[Expr] = []
        for elem in node.elements:
            renamed_elem = _rename_macro_names(elem, rename_map)
            if not isinstance(renamed_elem, Expr):
                raise TypeError(f"Expected Expr for element in TupleExpr, got {type(renamed_elem).__name__}")
            elements.append(renamed_elem)
        return TupleExpr(
            elements=elements
        )
    if isinstance(node, DictExpr):
        keys: List[Expr] = []
        for key in node.keys:
            renamed_key = _rename_macro_names(key, rename_map)
            if not isinstance(renamed_key, Expr):
                raise TypeError(f"Expected Expr for key in DictExpr, got {type(renamed_key).__name__}")
            keys.append(renamed_key)
        values: List[Expr] = []
        for value in node.values:
            renamed_value = _rename_macro_names(value, rename_map)
            if not isinstance(renamed_value, Expr):
                raise TypeError(f"Expected Expr for value in DictExpr, got {type(renamed_value).__name__}")
            values.append(renamed_value)
        return DictExpr(
            keys=keys,
            values=values
        )
    if isinstance(node, SetExpr):
        elements: List[Expr] = []
        for elem in node.elements:
            renamed_elem = _rename_macro_names(elem, rename_map)
            if not isinstance(renamed_elem, Expr):
                raise TypeError(f"Expected Expr for element in SetExpr, got {type(renamed_elem).__name__}")
            elements.append(renamed_elem)
        return SetExpr(
            elements=elements
        )
    if isinstance(node, Slice):
        lower = _rename_macro_names(node.lower, rename_map) if node.lower is not None else None
        upper = _rename_macro_names(node.upper, rename_map) if node.upper is not None else None
        step = _rename_macro_names(node.step, rename_map) if node.step is not None else None
        
        if lower is not None and not isinstance(lower, Expr):
            raise TypeError(f"Expected Expr for lower in Slice, got {type(lower).__name__}")
        if upper is not None and not isinstance(upper, Expr):
            raise TypeError(f"Expected Expr for upper in Slice, got {type(upper).__name__}")
        if step is not None and not isinstance(step, Expr):
            raise TypeError(f"Expected Expr for step in Slice, got {type(step).__name__}")
        return Slice(
            lower=lower,
            upper=upper,
            step=step,
        )
    if isinstance(node, BlockExpr):
        body: List[Stmt] = []
        for stmt in node.body:
            renamed_stmt = _rename_macro_names(stmt, rename_map)
            if isinstance(renamed_stmt, list):
                body.extend(renamed_stmt)
            elif isinstance(renamed_stmt, Stmt):
                body.append(renamed_stmt)
            else:
                raise TypeError(f"Expected Stmt or List[Stmt], got {type(renamed_stmt).__name__}")
        
        return BlockExpr(
            body=body
        )
    if isinstance(node, BlockStmt):
        body: List[Stmt] = []
        for stmt in node.body:
            renamed_stmt = _rename_macro_names(stmt, rename_map)
            if isinstance(renamed_stmt, list):
                body.extend(renamed_stmt)
            elif isinstance(renamed_stmt, Stmt):
                body.append(renamed_stmt)
            else:
                raise TypeError(f"Expected Stmt or List[Stmt], got {type(renamed_stmt).__name__}")
        return BlockStmt(
            body=body
        )
    return node


def _flatten_statements(statements: List[Stmt | List[Stmt]]) -> List[Stmt]:
    flattened: List[Stmt] = []
    for stmt in statements:
        if isinstance(stmt, list):
            flattened.extend(stmt)
        else:
            flattened.append(stmt)
    return flattened


def _body_to_list(body: BodyStmt | None) -> List[Stmt]:
    if body is None:
        return []
    if isinstance(body, BlockStmt):
        return body.body
    return [body]


def _build_body_stmt(original: BodyStmt, statements: List[Stmt]) -> BodyStmt:
    if isinstance(original, BlockStmt):
        return BlockStmt(body=statements)
    if len(statements) == 1:
        return statements[0]
    return BlockStmt(body=statements)


def _build_optional_body_stmt(original: BodyStmt | None, statements: List[Stmt]) -> BodyStmt | None:
    if original is None:
        return None
    return _build_body_stmt(original, statements)


def _is_assign_target(node: object) -> bool:
    return isinstance(node, (Name, Attribute, Subscript, TupleExpr, ListExpr))


def _is_body_stmt(node: object) -> bool:
    return isinstance(node, (BlockStmt, Stmt))


def _macro_param_kind_name(kind: MacroParamKind | None) -> str:
    if kind is None:
        return "any"
    return {
        MacroParamKind.EXPRESSION: "Expr",
        MacroParamKind.STATEMENT: "Stmt",
        MacroParamKind.IDENTIFIER: "Identifier",
        MacroParamKind.CODE: "Code",
    }[kind]


def _validate_macro_args(macro: MacroDef, args: List[Expr]) -> None:
    for param, arg in zip(macro.params, args):
        if param.type == MacroParamKind.IDENTIFIER:
            if not isinstance(arg, Name):
                raise SyntaxError(
                    f"Macro {macro.name} parameter '{param.name}' expects Identifier, got {type(arg).__name__}"
                )
        elif param.type in (MacroParamKind.STATEMENT, MacroParamKind.CODE):
            if not isinstance(arg, BlockExpr):
                raise SyntaxError(
                    f"Macro {macro.name} parameter '{param.name}' expects {param.type.name.title()}, got {type(arg).__name__}"
                )
        elif param.type == MacroParamKind.EXPRESSION:
            if isinstance(arg, BlockExpr):
                raise SyntaxError(
                    f"Macro {macro.name} parameter '{param.name}' expects Expr, got BlockExpr"
                )


def _substitute_macro_target(target: AssignTarget, arg_map: Dict[str, Expr | BlockExpr], macros: Dict[str, MacroDef] | None = None) -> AssignTarget:
    if isinstance(target, Name):
        result = _substitute_macro_expr(target, arg_map, macros)
        if not isinstance(result, Name):
            raise TypeError(f"Expected Name for assignment target, got {type(result).__name__}")
        return result
    if isinstance(target, Attribute):
        result = _substitute_macro_expr(target, arg_map, macros)
        if not isinstance(result, Attribute):
            raise TypeError(f"Expected Attribute for assignment target, got {type(result).__name__}")
        return result
    if isinstance(target, Subscript):
        result = _substitute_macro_expr(target, arg_map, macros)
        if not isinstance(result, Subscript):
            raise TypeError(f"Expected Subscript for assignment target, got {type(result).__name__}")
        return result
    elements_exprs: List[Expr] = []
    for elem in target.elements:
        if not _is_assign_target(elem):
            raise TypeError(f"Expected AssignTarget or Expr for tuple/list element, got {type(elem).__name__}")
        substituted_elem = _substitute_macro_target(elem, arg_map, macros)
        elements_exprs.append(substituted_elem)
    if isinstance(target, TupleExpr):
        return TupleExpr(elements=elements_exprs)
    return ListExpr(elements=elements_exprs)


def _substitute_macro_statement(node: Stmt, arg_map: Dict[str, Expr | BlockExpr], macros: Dict[str, MacroDef] | None = None) -> Stmt:
    if isinstance(node, ExprStmt) and isinstance(node.value, Name) and node.value.id in arg_map:
        replacement = arg_map[node.value.id]
        if isinstance(replacement, BlockExpr):
            return BlockStmt(body=replacement.body)
        return ExprStmt(value=replacement)
        
    if isinstance(node, ExprStmt):
        return ExprStmt(value=_substitute_macro_expr(node.value, arg_map, macros))
    if isinstance(node, Assign):
        return Assign(
            target=_substitute_macro_target(node.target, arg_map, macros),
            value=_substitute_macro_expr(node.value, arg_map, macros),
        )
    if isinstance(node, AugAssign):
        return AugAssign(
            target=_substitute_macro_target(node.target, arg_map, macros),
            op=node.op,
            value=_substitute_macro_expr(node.value, arg_map, macros),
        )
    if isinstance(node, If):
        body_stmts = _flatten_statements([_substitute_macro_statement(stmt, arg_map, macros) for stmt in _body_to_list(node.body)])
        orelse_stmts = _flatten_statements([_substitute_macro_statement(stmt, arg_map, macros) for stmt in _body_to_list(node.orelse)])
        return If(
            test=_substitute_macro_expr(node.test, arg_map, macros),
            body=_build_body_stmt(node.body, body_stmts),
            orelse=_substitute_macro_statement(node.orelse, arg_map, macros) if isinstance(node.orelse, If) else _build_optional_body_stmt(node.orelse, orelse_stmts),
        )
    if isinstance(node, While):
        body_stmts = _flatten_statements([_substitute_macro_statement(stmt, arg_map, macros) for stmt in _body_to_list(node.body)])
        orelse_stmts = _flatten_statements([_substitute_macro_statement(stmt, arg_map, macros) for stmt in _body_to_list(node.orelse)])
        return While(
            test=_substitute_macro_expr(node.test, arg_map, macros),
            body=_build_body_stmt(node.body, body_stmts),
            orelse=_build_optional_body_stmt(node.orelse, orelse_stmts),
        )
    if isinstance(node, For):
        body_stmts = _flatten_statements([_substitute_macro_statement(stmt, arg_map, macros) for stmt in _body_to_list(node.body)])
        target = node.target
        if isinstance(target, Name):
            target = _substitute_macro_expr(target, arg_map, macros)
            if not isinstance(target, Name):
                raise TypeError(f"Expected Name for for-loop target, got {type(target).__name__}")
        else:
            elements_exprs: List[Expr] = []
            for elem in target.elements:
                if not _is_assign_target(elem):
                    raise TypeError(f"Expected AssignTarget or Expr for tuple/list element, got {type(elem).__name__}")
                substituted_elem = _substitute_macro_target(elem, arg_map, macros)
                elements_exprs.append(substituted_elem)
            if isinstance(target, TupleExpr):
                target = TupleExpr(elements=elements_exprs)
            else:
                target = ListExpr(elements=elements_exprs)
        return For(
            target=target,
            iter=_substitute_macro_expr(node.iter, arg_map, macros),
            body=_build_body_stmt(node.body, body_stmts),
        )
    if isinstance(node, With):
        body_stmts = _flatten_statements([_substitute_macro_statement(stmt, arg_map, macros) for stmt in _body_to_list(node.body)])
        return With(
            context_expr=_substitute_macro_expr(node.context_expr, arg_map, macros),
            alias=node.alias,
            body=_build_body_stmt(node.body, body_stmts),
        )
    if isinstance(node, FunctionDef):
        body_stmts = _flatten_statements([_substitute_macro_statement(stmt, arg_map, macros) for stmt in _body_to_list(node.body)])
        return FunctionDef(
            name=node.name,
            args=node.args,
            body=_build_body_stmt(node.body, body_stmts),
            decorators=node.decorators,
            is_async=node.is_async,
            return_type=node.return_type,
        )
    return node


def _substitute_macro_expr(node: Expr, arg_map: Dict[str, Expr | BlockExpr], macros: Dict[str, MacroDef] | None = None) -> Expr:
    if isinstance(node, Name):
        if node.id in arg_map:
            replacement = arg_map[node.id]
            if isinstance(replacement, BlockExpr):
                raise SyntaxError(f"Block argument {node.id} cannot be used as expression")
            return copy.deepcopy(replacement)
        return node
    if isinstance(node, BinOp):
        return BinOp(
            left=_substitute_macro_expr(node.left, arg_map, macros),
            op=node.op,
            right=_substitute_macro_expr(node.right, arg_map, macros),
        )
    if isinstance(node, UnaryOp):
        return UnaryOp(op=node.op, operand=_substitute_macro_expr(node.operand, arg_map, macros))
    if isinstance(node, BoolOp):
        return BoolOp(op=node.op, values=[_substitute_macro_expr(value, arg_map, macros) for value in node.values])
    if isinstance(node, Compare):
        return Compare(
            left=_substitute_macro_expr(node.left, arg_map, macros),
            ops=node.ops,
            comparators=[_substitute_macro_expr(comp, arg_map, macros) for comp in node.comparators],
        )
    if isinstance(node, MacroCall):
        return MacroCall(
            name=node.name,
            args=[_substitute_macro_expr(arg, arg_map, macros) for arg in node.args],
        )
    if isinstance(node, Call):
        return Call(
            func=_substitute_macro_expr(node.func, arg_map, macros), 
            args=[_substitute_macro_expr(arg, arg_map, macros) for arg in node.args],
            keywords=node.keywords,
            starargs=node.starargs,
            kwargs=node.kwargs,
        )
    if isinstance(node, ListExpr):
        return ListExpr(elements=[_substitute_macro_expr(elem, arg_map, macros) for elem in node.elements])
    if isinstance(node, TupleExpr):
        return TupleExpr(elements=[_substitute_macro_expr(elem, arg_map, macros) for elem in node.elements])
    if isinstance(node, DictExpr):
        return DictExpr(keys=[_substitute_macro_expr(key, arg_map, macros) for key in node.keys], values=[_substitute_macro_expr(value, arg_map, macros) for value in node.values])
    if isinstance(node, SetExpr):
        return SetExpr(elements=[_substitute_macro_expr(elem, arg_map, macros) for elem in node.elements])
    if isinstance(node, Subscript):
        return Subscript(value=_substitute_macro_expr(node.value, arg_map, macros), slice=_substitute_macro_expr(node.slice, arg_map, macros))
    if isinstance(node, Slice):
        return Slice(
            lower=_substitute_macro_expr(node.lower, arg_map, macros) if node.lower is not None else None,
            upper=_substitute_macro_expr(node.upper, arg_map, macros) if node.upper is not None else None,
            step=_substitute_macro_expr(node.step, arg_map, macros) if node.step is not None else None,
        )
    if isinstance(node, Lambda):
        return Lambda(
            args=node.args,
            return_type=node.return_type,
            expr=_substitute_macro_expr(node.expr, arg_map, macros),
        )
    if isinstance(node, Yield):
        return Yield(value=_substitute_macro_expr(node.value, arg_map, macros) if node.value is not None else None)
    if isinstance(node, ListComp):
        return ListComp(
            elt=_substitute_macro_expr(node.elt, arg_map, macros),
            generators=[_substitute_macro_comprehension(gen, arg_map, macros) for gen in node.generators],
        )
    if isinstance(node, SetComp):
        return SetComp(
            elt=_substitute_macro_expr(node.elt, arg_map, macros),
            generators=[_substitute_macro_comprehension(gen, arg_map, macros) for gen in node.generators],
        )
    if isinstance(node, DictComp):
        return DictComp(
            key=_substitute_macro_expr(node.key, arg_map, macros),
            value=_substitute_macro_expr(node.value, arg_map, macros),
            generators=[_substitute_macro_comprehension(gen, arg_map, macros) for gen in node.generators],
        )
    if isinstance(node, GeneratorExp):
        return GeneratorExp(
            elt=_substitute_macro_expr(node.elt, arg_map, macros),
            generators=[_substitute_macro_comprehension(gen, arg_map, macros) for gen in node.generators],
        )
    if isinstance(node, Attribute):
        value = _substitute_macro_expr(node.value, arg_map, macros)
        if node.attr == "items" and isinstance(value, ListExpr):
            return Call(
                func=Attribute(value=DictExpr(keys=[Constant(value=_expr_to_source(elem, macros)) for elem in value.elements], values=[copy.deepcopy(elem) for elem in value.elements]), attr="items"),
                args=[],
                keywords=[],
                starargs=[],
                kwargs=[],
            )
        return Attribute(value=value, attr=node.attr)
    return node


def _substitute_macro_comprehension(node: Comprehension, arg_map: Dict[str, Expr | BlockExpr], macros: Dict[str, MacroDef] | None = None) -> Comprehension:
    target = node.target
    if isinstance(target, (Name, Attribute, Subscript)):
        target = _substitute_macro_expr(target, arg_map, macros)
    elif isinstance(target, TupleExpr):
        elements: Sequence[Expr] = []
        for elem in target.elements:
            if not _is_assign_target(elem):
                raise TypeError(f"Expected AssignTarget or Expr for tuple element, got {type(elem).__name__}")
            substituted_elem = _substitute_macro_target(elem, arg_map, macros)
            elements.append(substituted_elem)
        target = TupleExpr(
            elements=elements
        )
    else:
        elements: Sequence[Expr] = []
        for elem in target.elements:
            if not _is_assign_target(elem):
                raise TypeError(f"Expected AssignTarget or Expr for list element, got {type(elem).__name__}")
            substituted_elem = _substitute_macro_target(elem, arg_map, macros)
            elements.append(substituted_elem)
        target = ListExpr(
            elements=elements
        )
    if not _is_assign_target(target):
        raise TypeError(f"Expected AssignTarget for comprehension target, got {type(target).__name__}")
    return Comprehension(
        target=target,
        iter=_substitute_macro_expr(node.iter, arg_map, macros),
        ifs=[_substitute_macro_expr(if_expr, arg_map, macros) for if_expr in node.ifs],
    )


def _expr_to_source(node: Expr, macros: Dict[str, MacroDef] | None = None) -> str:
    if macros is None:
        macros = {}
    try:
        return ast.unparse(_compile_expr(node, macros))
    except Exception:
        if isinstance(node, Name):
            return node.id
        if isinstance(node, Attribute):
            return f"{_expr_to_source(node.value, macros)}.{node.attr}"
        if isinstance(node, Constant):
            return repr(node.value)
        if isinstance(node, FString):
            return node.value
        if isinstance(node, TupleExpr):
            return "(" + ", ".join(_expr_to_source(elem, macros) for elem in node.elements) + ")"
        if isinstance(node, ListExpr):
            return "[" + ", ".join(_expr_to_source(elem, macros) for elem in node.elements) + "]"
        return str(node)


def _compile_hooks_dict() -> ast.stmt:
    return ast.Assign(
        targets=[ast.Name(id="_pypp_hooks", ctx=ast.Store())],
        value=ast.Dict(keys=[], values=[]),
    )


def _compile_apply_hook_function() -> ast.stmt:
    return ast.FunctionDef(
        name="_pypp_apply_hook",
        args=ast.arguments(
            posonlyargs=[],
            args=[ast.arg(arg="name"), ast.arg(arg="value")],
            kwonlyargs=[],
            kw_defaults=[],
            defaults=[],
        ),
        body=[
            ast.If(
                test=ast.Compare(
                    left=ast.Name(id="name", ctx=ast.Load()),
                    ops=[ast.In()],
                    comparators=[ast.Name(id="_pypp_hooks", ctx=ast.Load())],
                ),
                body=[
                    ast.Return(
                        value=ast.Call(
                            func=ast.Subscript(
                                value=ast.Name(id="_pypp_hooks", ctx=ast.Load()),
                                slice=ast.Name(id="name", ctx=ast.Load()),
                                ctx=ast.Load(),
                            ),
                            args=[ast.Name(id="value", ctx=ast.Load())],
                            keywords=[],
                        )
                    )
                ],
                orelse=[ast.Return(value=ast.Name(id="value", ctx=ast.Load()))],
            )
        ],
        decorator_list=[],
        returns=None,
    )


def _flatten_compiled_stmt(compiled_stmt: ast.stmt | list[ast.stmt] | ast.Module) -> List[ast.stmt]:
    if isinstance(compiled_stmt, ast.Module):
        return list(compiled_stmt.body)
    if isinstance(compiled_stmt, list):
        return compiled_stmt
    return [compiled_stmt]


def _compile_stmt_list(node: Stmt | List[Stmt], use_hooks: bool, in_class: bool = False, class_name: str | None = None, macros: Dict[str, MacroDef] | None = None, source_base_path: Path | None = None) -> List[ast.stmt]:
    if macros is None:
        macros = {}
    if isinstance(node, list):
        compiled: List[ast.stmt] = []
        for stmt in node:
            compiled.extend(_compile_stmt_list(stmt, use_hooks, in_class=in_class, class_name=class_name, macros=macros, source_base_path=source_base_path))
        return compiled
    if isinstance(node, MacroDef):
        return []
    if isinstance(node, ExprStmt) and isinstance(node.value, MacroCall):
        return _compile_macro_call(node.value, macros, use_hooks)
    if isinstance(node, MacroCall):
        return _compile_macro_call(node, macros, use_hooks)
    if isinstance(node, When):
        hook_name = f"__pypp_hook_{class_name + '_' if class_name else ''}{node.var}"
        body_statements: List[ast.stmt] = []
        for stmt in _body_to_list(node.body):
            body_statements.extend(_compile_stmt_list(stmt, use_hooks=False, in_class=in_class, class_name=class_name, macros=macros, source_base_path=source_base_path))
        func_def = ast.FunctionDef(
            name=hook_name,
            args=ast.arguments(
                posonlyargs=[],
                args=[ast.arg(arg=node.param)],
                kwonlyargs=[],
                kw_defaults=[],
                defaults=[],
            ),
            body=body_statements,
            decorator_list=[],
            returns=None,
        )
        statements: List[ast.stmt] = [func_def]
        register_target = node.var if not class_name else f"{class_name}.{node.var}"
        register = ast.Assign(
            targets=[
                ast.Subscript(
                    value=ast.Name(id="_pypp_hooks", ctx=ast.Load()),
                    slice=ast.Constant(value=register_target),
                    ctx=ast.Store(),
                )
            ],
            value=ast.Name(id=hook_name, ctx=ast.Load()),
        )
        statements.append(register)
        return statements
    compiled: List[ast.stmt] = []
    stmt = _compile_stmt(node, use_hooks, in_class=in_class, class_name=class_name, macros=macros, source_base_path=source_base_path)
    compiled.extend(_flatten_compiled_stmt(stmt))
    return compiled


def _compile_stmt(node: Stmt, use_hooks: bool, in_class: bool = False, class_name: str | None = None, macros: Dict[str, MacroDef] | None = None, source_base_path: Path | None = None) -> ast.stmt | list[ast.stmt]:
    if macros is None:
        macros = {}
    if isinstance(node, BlockStmt):
        compiled: List[ast.stmt] = []
        for stmt in node.body:
            compiled.extend(_compile_stmt_list(stmt, use_hooks, in_class=in_class, class_name=class_name, macros=macros, source_base_path=source_base_path))
        return compiled
    if isinstance(node, ExprStmt) and isinstance(node.value, MacroCall):
        return _compile_macro_call(node.value, macros, use_hooks)
    if isinstance(node, ExprStmt):
        return ast.Expr(value=_compile_expr(node.value, macros))
    if isinstance(node, MacroCall):
        return _compile_macro_call(node, macros, use_hooks)
    if isinstance(node, Use):
        alias = node.alias
        source_str = _use_source_to_string(node.source)
        if alias is None:
            alias = source_str
        if node.source.is_string:
            imported_path = Path(source_str)
            if not imported_path.is_absolute() and source_base_path is not None:
                imported_path = source_base_path / imported_path
            if imported_path.suffix == ".pypp" or imported_path.with_suffix(".pypp").exists():
                if imported_path.suffix != ".pypp":
                    imported_path = imported_path.with_suffix(".pypp")
                return ast.Assign(
                    targets=[ast.Name(id=alias, ctx=ast.Store())],
                    value=ast.Call(
                        func=ast.Name(id="__pypp_import_pypp_file", ctx=ast.Load()),
                        args=[ast.Constant(value=str(imported_path)), ast.Constant(value=alias)],
                        keywords=[],
                    ),
                )
            if imported_path.suffix == ".py" or imported_path.with_suffix(".py").exists():
                if imported_path.suffix != ".py":
                    imported_path = imported_path.with_suffix(".py")
                return ast.Assign(
                    targets=[ast.Name(id=alias, ctx=ast.Store())],
                    value=ast.Call(
                        func=ast.Name(id="__pypp_import_file", ctx=ast.Load()),
                        args=[ast.Constant(value=str(imported_path)), ast.Constant(value=alias)],
                        keywords=[],
                    ),
                )
            return ast.Import(names=[ast.alias(name=source_str, asname=None if source_str == alias else alias)])
        return ast.Import(names=[ast.alias(name=source_str, asname=None if source_str == alias else alias)])
    if isinstance(node, FromUse):
        source_str = _use_source_to_string(node.source)
        if node.source.is_string:
            imported_path = Path(source_str)
            if not imported_path.is_absolute() and source_base_path is not None:
                imported_path = source_base_path / imported_path
            if imported_path.suffix == ".pypp" or imported_path.with_suffix(".pypp").exists():
                if imported_path.suffix != ".pypp":
                    imported_path = imported_path.with_suffix(".pypp")
                module_alias = _get_default_alias(source_str)
                statements: List[ast.stmt] = [
                    ast.Assign(
                        targets=[ast.Name(id=module_alias, ctx=ast.Store())],
                        value=ast.Call(
                            func=ast.Name(id="__pypp_import_pypp_file", ctx=ast.Load()),
                            args=[ast.Constant(value=str(imported_path)), ast.Constant(value=module_alias)],
                            keywords=[],
                        ),
                    )
                ]
                for name, _, alias in node.names:
                    target_name = alias or name
                    statements.append(
                        ast.Assign(
                            targets=[ast.Name(id=target_name, ctx=ast.Store())],
                            value=ast.Attribute(
                                value=ast.Name(id=module_alias, ctx=ast.Load()),
                                attr=name,
                                ctx=ast.Load(),
                            ),
                        )
                    )
                return statements
            if imported_path.suffix == ".py" or imported_path.with_suffix(".py").exists():
                if imported_path.suffix != ".py":
                    imported_path = imported_path.with_suffix(".py")
                module_alias = _get_default_alias(source_str)
                statements = [
                    ast.Assign(
                        targets=[ast.Name(id=module_alias, ctx=ast.Store())],
                        value=ast.Call(
                            func=ast.Name(id="__pypp_import_file", ctx=ast.Load()),
                            args=[ast.Constant(value=str(imported_path)), ast.Constant(value=module_alias)],
                            keywords=[],
                        ),
                    )
                ]
                for name, _, alias in node.names:
                    target_name = alias or name
                    statements.append(
                        ast.Assign(
                            targets=[ast.Name(id=target_name, ctx=ast.Store())],
                            value=ast.Attribute(
                                value=ast.Name(id=module_alias, ctx=ast.Load()),
                                attr=name,
                                ctx=ast.Load(),
                            ),
                        )
                    )
                return statements
        return ast.ImportFrom(
            module=source_str,
            names=[ast.alias(name=name, asname=alias) for name, _, alias in node.names],
            level=0,
        )
    if isinstance(node, AnnAssign):
        target_expr = _compile_target(node.target)
        if not isinstance(target_expr, (ast.Name, ast.Attribute, ast.Subscript)):
            raise TypeError(f"Expected ast.Name, ast.Attribute, or ast.Subscript for assignment target, got {type(target_expr).__name__}")
        return ast.AnnAssign(
            target=target_expr,
            annotation=_compile_expr(node.annotation, macros),
            value=_compile_expr(node.value, macros) if node.value is not None else None,
            simple=1 if node.simple else 0,
        )
    if isinstance(node, Assign):
        value = _compile_expr(node.value, macros)
        target_expr = _compile_target(node.target)
        hook_name = None
        if use_hooks:
            if isinstance(node.target, Name):
                hook_name = node.target.id
            elif isinstance(node.target, Attribute) and class_name is not None:
                if isinstance(node.target.value, Name) and node.target.value.id == "self":
                    hook_name = f"{class_name}.{node.target.attr}"
        if hook_name is not None and not (in_class and isinstance(node.target, Attribute)):
            value = ast.Call(
                func=ast.Name(id="_pypp_apply_hook", ctx=ast.Load()),
                args=[ast.Constant(value=hook_name), value],
                keywords=[],
            )
        return ast.Assign(
            targets=[target_expr],
            value=value,
        )
    if isinstance(node, ClassDef):
        bases: List[ast.expr] = [ast.Name(id=_expr_to_source(base, macros), ctx=ast.Load()) for base in node.bases]
        body: List[ast.stmt] = []
        when_fields: List[str] = []
        for stmt in node.body.body:
            if isinstance(stmt, When):
                when_fields.append(stmt.var)
            body.extend(_compile_stmt_list(stmt, use_hooks=use_hooks, in_class=True, class_name=node.name, macros=macros))
        if when_fields:
            body.append(_compile_class_setattr(node.name, when_fields))
        if not body:
            body = [ast.Pass()]
        return ast.ClassDef(
            name=node.name,
            bases=bases,
            keywords=[],
            body=body,
            decorator_list=[_compile_expr(decorator, macros) for decorator in node.decorators],
        )
    if isinstance(node, AugAssign):
        value = _compile_expr(node.value, macros)
        target_expr = _compile_target(node.target)
        hook_name = None
        if isinstance(node.target, Name):
            hook_name = node.target.id
        elif isinstance(node.target, Attribute) and class_name is not None:
            if isinstance(node.target.value, Name) and node.target.value.id == "self":
                hook_name = f"{class_name}.{node.target.attr}"
        if use_hooks and hook_name is not None:
            compute_value = ast.BinOp(
                left=target_expr,
                op=_compile_augop(node.op),
                right=value,
            )
            value = ast.Call(
                func=ast.Name(id="_pypp_apply_hook", ctx=ast.Load()),
                args=[ast.Constant(value=hook_name), compute_value],
                keywords=[],
            )
            return ast.Assign(
                targets=[target_expr],
                value=value,
            )
        if not isinstance(target_expr, (ast.Name, ast.Attribute, ast.Subscript)):
            raise TypeError(f"Expected ast.Name, ast.Attribute, or ast.Subscript for assignment target, got {type(target_expr).__name__}")
        return ast.AugAssign(
            target=target_expr,
            op=_compile_augop(node.op),
            value=value,
        )
    if isinstance(node, If):
        body_stmts: List[ast.stmt] = []
        for stmt in _iter_stmt_body(node.body):
            compiled_stmt = _compile_stmt(stmt, use_hooks, macros=macros)
            if isinstance(compiled_stmt, ast.Module):
                body_stmts.extend(compiled_stmt.body)
            elif isinstance(compiled_stmt, list):
                body_stmts.extend(compiled_stmt)
            else:
                body_stmts.append(compiled_stmt)
        compiled_orelse: List[ast.stmt] = []
        for stmt in _iter_stmt_body(node.orelse):
            compiled_stmt = _compile_stmt(stmt, use_hooks, macros=macros)
            if isinstance(compiled_stmt, ast.Module):
                compiled_orelse.extend(compiled_stmt.body)
            elif isinstance(compiled_stmt, list):
                compiled_orelse.extend(compiled_stmt)
            else:
                compiled_orelse.append(compiled_stmt)
        return ast.If(
            test=_compile_expr(node.test, macros),
            body=body_stmts,
            orelse=compiled_orelse,
        )
    if isinstance(node, While):
        body_stmts: List[ast.stmt] = []
        for stmt in _iter_stmt_body(node.body):
            compiled_stmt = _compile_stmt(stmt, use_hooks, macros=macros)
            if isinstance(compiled_stmt, ast.Module):
                body_stmts.extend(compiled_stmt.body)
            elif isinstance(compiled_stmt, list):
                body_stmts.extend(compiled_stmt)
            else:
                body_stmts.append(compiled_stmt)
        return ast.While(
            test=_compile_expr(node.test, macros),
            body=body_stmts,
            orelse=[],
        )
    if isinstance(node, For):
        body_stmts: List[ast.stmt] = []
        for stmt in _iter_stmt_body(node.body):
            compiled_stmt = _compile_stmt(stmt, use_hooks, macros=macros)
            if isinstance(compiled_stmt, ast.Module):
                body_stmts.extend(compiled_stmt.body)
            elif isinstance(compiled_stmt, list):
                body_stmts.extend(compiled_stmt)
            else:
                body_stmts.append(compiled_stmt)
        return ast.For(
            target=_compile_target(node.target),
            iter=_compile_expr(node.iter, macros),
            body=body_stmts,
            orelse=[],
        )
    if isinstance(node, With):
        body_stmts: List[ast.stmt] = []
        for stmt in _iter_stmt_body(node.body):
            compiled_stmt = _compile_stmt(stmt, use_hooks, macros=macros)
            if isinstance(compiled_stmt, ast.Module):
                body_stmts.extend(compiled_stmt.body)
            elif isinstance(compiled_stmt, list):
                body_stmts.extend(compiled_stmt)
            else:
                body_stmts.append(compiled_stmt)
        return ast.With(
            items=[ast.withitem(
                context_expr=_compile_expr(node.context_expr, macros),
                optional_vars=ast.Name(id=node.alias, ctx=ast.Store()) if node.alias else None,
            )],
            body=body_stmts,
            type_comment=None,
        )
    if isinstance(node, Assert):
        return ast.Assert(
            test=_compile_expr(node.test, macros),
            msg=_compile_expr(node.msg, macros) if node.msg is not None else None,
        )
    if isinstance(node, Raise):
        return ast.Raise(
            exc=_compile_expr(node.exc, macros) if node.exc is not None else None,
            cause=None,
        )
    if isinstance(node, Try):
        body_stmts: List[ast.stmt] = []
        for stmt in _iter_stmt_body(node.body):
            compiled_stmt = _compile_stmt(stmt, use_hooks, macros=macros)
            body_stmts.extend(_flatten_compiled_stmt(compiled_stmt))
        handlers: List[ast.ExceptHandler] = []
        for handler in node.handlers:
            handler_body_stmts: List[ast.stmt] = []
            for stmt in _iter_stmt_body(handler.body):
                compiled_stmt = _compile_stmt(stmt, use_hooks, macros=macros)
                if isinstance(compiled_stmt, ast.Module):
                    handler_body_stmts.extend(compiled_stmt.body)
                elif isinstance(compiled_stmt, list):
                    handler_body_stmts.extend(compiled_stmt)
                else:
                    handler_body_stmts.append(compiled_stmt)
            handlers.append(ast.ExceptHandler(
                type=_compile_expr(handler.type, macros) if handler.type is not None else None,
                name=handler.name,
                body=handler_body_stmts,
            ))
        orelse_stmts: List[ast.stmt] = []
        for stmt in _iter_stmt_body(node.orelse):
            compiled_stmt = _compile_stmt(stmt, use_hooks, macros=macros)
            if isinstance(compiled_stmt, ast.Module):
                orelse_stmts.extend(compiled_stmt.body)
            elif isinstance(compiled_stmt, list):
                orelse_stmts.extend(compiled_stmt)
            else:
                orelse_stmts.append(compiled_stmt)
        
        finalbody_stmts: List[ast.stmt] = []
        for stmt in _iter_stmt_body(node.finalbody):
            compiled_stmt = _compile_stmt(stmt, use_hooks, macros=macros)
            if isinstance(compiled_stmt, ast.Module):
                finalbody_stmts.extend(compiled_stmt.body)
            elif isinstance(compiled_stmt, list):
                finalbody_stmts.extend(compiled_stmt)
            else:
                finalbody_stmts.append(compiled_stmt)    
        return ast.Try(
            body=body_stmts,
            handlers=handlers,
            orelse=orelse_stmts,
            finalbody=finalbody_stmts,
        )
    if isinstance(node, Del):
        return ast.Delete(targets=[_compile_del_target(target) for target in node.targets])
    if isinstance(node, Global):
        return ast.Global(names=node.names)
    if isinstance(node, Nonlocal):
        return ast.Nonlocal(names=node.names)
    if isinstance(node, Break):
        return ast.Break()
    if isinstance(node, Continue):
        return ast.Continue()
    if isinstance(node, Match):
        cases: List[ast.match_case] = []
        for case in node.cases:
            case_body: List[ast.stmt] = []
            for stmt in _iter_stmt_body(case.body):
                case_body.extend(_flatten_compiled_stmt(_compile_stmt(stmt, use_hooks, macros=macros)))
            cases.append(
                ast.match_case(
                    pattern=_compile_pattern(case.pattern),
                    guard=_compile_expr(case.guard, macros) if case.guard is not None else None,
                    body=case_body,
                )
            )
        return ast.Match(
            subject=_compile_expr(node.subject, macros),
            cases=cases,
        )
    if isinstance(node, MacroDef):
        # Macro definitions are compile-time constructs only. They are not emitted as
        # runtime Python code.
        return []
    if isinstance(node, FunctionDef):
        args: List[ast.arg] = []
        kwonlyargs: List[ast.arg] = []
        defaults: List[ast.expr] = []
        kw_defaults: List[ast.expr | None] = []
        vararg: ast.arg | None = None
        kwarg: ast.arg | None = None
        for arg_node in node.args:
            if arg_node.kind == ArgKind.POSITIONAL:
                args.append(ast.arg(arg=arg_node.name, annotation=_compile_expr(arg_node.annotation, macros) if arg_node.annotation is not None else None))
                if arg_node.default is not None:
                    defaults.append(_compile_expr(arg_node.default, macros))
            elif arg_node.kind == ArgKind.VARARG:
                vararg = ast.arg(arg=arg_node.name, annotation=_compile_expr(arg_node.annotation, macros) if arg_node.annotation is not None else None)
            elif arg_node.kind == ArgKind.KWARG:
                kwarg = ast.arg(arg=arg_node.name, annotation=_compile_expr(arg_node.annotation, macros) if arg_node.annotation is not None else None)
            elif arg_node.kind == ArgKind.KEYWORD_ONLY:
                kwonlyargs.append(ast.arg(arg=arg_node.name, annotation=_compile_expr(arg_node.annotation, macros) if arg_node.annotation is not None else None))
                kw_defaults.append(_compile_expr(arg_node.default, macros) if arg_node.default is not None else None)
        if in_class:
            if not args or args[0].arg != "self":
                args.insert(0, ast.arg(arg="self"))
        body: List[ast.stmt] = []
        for stmt in _iter_stmt_body(node.body):
            body.extend(_flatten_compiled_stmt(_compile_stmt(stmt, use_hooks, in_class=in_class, class_name=class_name, macros=macros)))
        return ast.AsyncFunctionDef(
                name=__compile_method_name(node.name, class_name) if in_class else node.name,
                args=ast.arguments(
                    posonlyargs=[],
                    args=args,
                    vararg=vararg,
                    kwonlyargs=kwonlyargs,
                    kw_defaults=kw_defaults,
                    kwarg=kwarg,
                    defaults=defaults,
                ),
                body=body,
                decorator_list=[_compile_expr(decorator, macros) for decorator in node.decorators],
                returns=_compile_expr(node.return_type, macros) if node.return_type is not None else None,
            ) if node.is_async else ast.FunctionDef(
                name=__compile_method_name(node.name, class_name) if in_class else node.name,
                args=ast.arguments(
                    posonlyargs=[],
                    args=args,
                    vararg=vararg,
                    kwonlyargs=kwonlyargs,
                    kw_defaults=kw_defaults,
                    kwarg=kwarg,
                    defaults=defaults,
                ),
                body=body,
                decorator_list=[_compile_expr(decorator, macros) for decorator in node.decorators],
                returns=_compile_expr(node.return_type, macros) if node.return_type is not None else None,
            )
    if isinstance(node, Return):
        return ast.Return(value=_compile_expr(node.value, macros) if node.value is not None else None)
    if isinstance(node, Pass):
        return ast.Pass()
    if isinstance(node, When):
        return ast.Pass()
    raise TypeError(f"Unsupported statement node: {type(node).__name__}")


def _compile_expr(node: Expr, macros: Dict[str, MacroDef] | None = None) -> ast.expr:
    if macros is None:
        macros = {}
    compile_expr: Callable[[Expr], ast.expr] = lambda expr: _compile_expr(expr, macros)
    if isinstance(node, Constant):
        return ast.Constant(value=node.value)
    if isinstance(node, FString):
        return ast.parse(node.value, mode="eval").body
    if isinstance(node, Name):
        return ast.Name(id=node.id, ctx=ast.Load())
    if isinstance(node, BinOp):
        left = compile_expr(node.left)
        right = compile_expr(node.right)
        binop_map: Dict[str, type[ast.operator]] = {
            "+": ast.Add,
            "-": ast.Sub,
            "*": ast.Mult,
            "/": ast.Div,
            "%": ast.Mod,
            "//": ast.FloorDiv,
            "**": ast.Pow,
            "&": ast.BitAnd,
            "|": ast.BitOr,
            "^": ast.BitXor,
            "<<": ast.LShift,
            ">>": ast.RShift,
        }
        op = binop_map[node.op]()
        return ast.BinOp(left=left, op=op, right=right)
    if isinstance(node, UnaryOp):
        operand = compile_expr(node.operand)
        if node.op == "+":
            op = ast.UAdd()
        elif node.op == "-":
            op = ast.USub()
        else:
            op = ast.Not()
        return ast.UnaryOp(op=op, operand=operand)
    if isinstance(node, BoolOp):
        return ast.BoolOp(
            op=ast.And() if node.op == "and" else ast.Or(),
            values=[compile_expr(value) for value in node.values],
        )
    if isinstance(node, Compare):
        return ast.Compare(
            left=compile_expr(node.left),
            ops=[
                {
                    "==": ast.Eq,
                    "!=": ast.NotEq,
                    "<": ast.Lt,
                    ">": ast.Gt,
                    "<=": ast.LtE,
                    ">=": ast.GtE,
                    "is": ast.Is,
                    "is not": ast.IsNot,
                    "in": ast.In,
                    "not in": ast.NotIn,
                }[op]()
                for op in node.ops
            ],
            comparators=[compile_expr(comp) for comp in node.comparators],
        )
    if isinstance(node, BinOp):
        left = compile_expr(node.left)
        right = compile_expr(node.right)
        binop_map: Dict[str, type[ast.operator]] = {
            "+": ast.Add,
            "-": ast.Sub,
            "*": ast.Mult,
            "/": ast.Div,
            "%": ast.Mod,
            "//": ast.FloorDiv,
        }
        op = binop_map[node.op]()
        return ast.BinOp(left=left, op=op, right=right)
    if isinstance(node, MacroCall):
        return _compile_macro_expr(node, macros)
    if isinstance(node, Call):
        compiled_args: List[ast.expr] = [compile_expr(arg) for arg in node.args]
        compiled_args.extend(
            ast.Starred(value=compile_expr(stararg), ctx=ast.Load()) for stararg in node.starargs
        )
        compiled_keywords: List[ast.keyword] = [
            ast.keyword(arg=arg_name, value=compile_expr(arg_value))
            for arg_name, arg_value in node.keywords
        ]
        compiled_keywords.extend(
            ast.keyword(arg=None, value=compile_expr(kwargs)) for kwargs in node.kwargs
        )
        return ast.Call(
            func=compile_expr(node.func),
            args=compiled_args,
            keywords=compiled_keywords,
        )
    if isinstance(node, IfExpr):
        return ast.IfExp(
            test=compile_expr(node.test),
            body=compile_expr(node.body),
            orelse=compile_expr(node.orelse),
        )
    if isinstance(node, ListExpr):
        return ast.List(elts=[compile_expr(elem) for elem in node.elements], ctx=ast.Load())
    if isinstance(node, TupleExpr):
        return ast.Tuple(elts=[compile_expr(elem) for elem in node.elements], ctx=ast.Load())
    if isinstance(node, DictExpr):
        return ast.Dict(keys=[compile_expr(key) for key in node.keys], values=[compile_expr(value) for value in node.values])
    if isinstance(node, SetExpr):
        return ast.Set(elts=[compile_expr(elem) for elem in node.elements])
    if isinstance(node, Subscript):
        return ast.Subscript(value=compile_expr(node.value), slice=compile_expr(node.slice), ctx=ast.Load())
    if isinstance(node, Slice):
        return ast.Slice(lower=compile_expr(node.lower) if node.lower is not None else None, upper=compile_expr(node.upper) if node.upper is not None else None, step=compile_expr(node.step) if node.step is not None else None)
    if isinstance(node, Lambda):
        args: List[ast.arg] = []
        kwonlyargs: List[ast.arg] = []
        defaults: List[ast.expr] = []
        kw_defaults: List[ast.expr | None] = []
        vararg: ast.arg | None = None
        kwarg: ast.arg | None = None
        for arg_node in node.args:
            if arg_node.kind == ArgKind.POSITIONAL:
                args.append(ast.arg(arg=arg_node.name, annotation=_compile_expr(arg_node.annotation, macros) if arg_node.annotation is not None else None))
                if arg_node.default is not None:
                    defaults.append(_compile_expr(arg_node.default, macros))
            elif arg_node.kind == ArgKind.VARARG:
                vararg = ast.arg(arg=arg_node.name, annotation=_compile_expr(arg_node.annotation, macros) if arg_node.annotation is not None else None)
            elif arg_node.kind == ArgKind.KWARG:
                kwarg = ast.arg(arg=arg_node.name, annotation=_compile_expr(arg_node.annotation, macros) if arg_node.annotation is not None else None)
            elif arg_node.kind == ArgKind.KEYWORD_ONLY:
                kwonlyargs.append(ast.arg(arg=arg_node.name, annotation=_compile_expr(arg_node.annotation, macros) if arg_node.annotation is not None else None))
                kw_defaults.append(_compile_expr(arg_node.default, macros) if arg_node.default is not None else None)
        return ast.Lambda(
            args=ast.arguments(
                posonlyargs=[],
                args=args,
                kwonlyargs=kwonlyargs,
                kw_defaults=kw_defaults,
                vararg=vararg,
                kwarg=kwarg,
                defaults=defaults,
            ),
            body=compile_expr(node.expr),
        )
    if isinstance(node, Yield):
        return ast.Yield(value=compile_expr(node.value) if node.value is not None else None)
    if isinstance(node, ListComp):
        return ast.ListComp(
            elt=compile_expr(node.elt),
            generators=[_compile_comprehension(gen, macros) for gen in node.generators],
        )
    if isinstance(node, SetComp):
        return ast.SetComp(
            elt=compile_expr(node.elt),
            generators=[_compile_comprehension(gen, macros) for gen in node.generators],
        )
    if isinstance(node, DictComp):
        return ast.DictComp(
            key=compile_expr(node.key),
            value=compile_expr(node.value),
            generators=[_compile_comprehension(gen, macros) for gen in node.generators],
        )
    if isinstance(node, GeneratorExp):
        return ast.GeneratorExp(
            elt=compile_expr(node.elt),
            generators=[_compile_comprehension(gen, macros) for gen in node.generators],
        )
    if isinstance(node, Attribute):
        return ast.Attribute(
            value=compile_expr(node.value),
            attr=node.attr,
            ctx=ast.Load(),
        )
    if isinstance(node, Await):
        return ast.Await(value=compile_expr(node.value))
    raise TypeError(f"Unsupported expression node: {type(node).__name__}")


def _compile_del_target(node: Expr) -> ast.expr:
    if isinstance(node, Name):
        return ast.Name(id=node.id, ctx=ast.Del())
    if isinstance(node, Attribute):
        return ast.Attribute(
            value=_compile_expr(node.value),
            attr=node.attr,
            ctx=ast.Del(),
        )
    if isinstance(node, Subscript):
        return ast.Subscript(
            value=_compile_expr(node.value),
            slice=_compile_expr(node.slice),
            ctx=ast.Del(),
        )
    raise TypeError(f"Unsupported del target: {type(node).__name__}")


def _compile_target(node: AssignTarget) -> ast.expr:
    if isinstance(node, Name):
        return ast.Name(id=node.id, ctx=ast.Store())
    elif isinstance(node, Attribute):
        return ast.Attribute(
            value=_compile_expr(node.value),
            attr=node.attr,
            ctx=ast.Store(),
        )
    elif isinstance(node, Subscript):
        return ast.Subscript(
            value=_compile_expr(node.value),
            slice=_compile_expr(node.slice),
            ctx=ast.Store(),
        )
    else:
        elements: List[ast.expr] = []
        for elem in node.elements:
            if not _is_assign_target(elem):
                raise TypeError(f"Expected AssignTarget for unpacking element, got {type(elem).__name__}")
            element_expr = _compile_target(elem)
            elements.append(element_expr)
        if isinstance(node, TupleExpr):
            return ast.Tuple(elts=elements, ctx=ast.Store())
        return ast.List(elts=elements, ctx=ast.Store())


def _compile_comprehension(node: Comprehension, macros: Dict[str, MacroDef] | None = None) -> ast.comprehension:
    return ast.comprehension(
        target=_compile_target(node.target),
        iter=_compile_expr(node.iter, macros),
        ifs=[_compile_expr(if_expr, macros) for if_expr in node.ifs],
        is_async=0,
    )


def _compile_pattern(node: Pattern, macros: Dict[str, MacroDef] | None = None) -> ast.pattern:
    if isinstance(node, MatchValue):
        return ast.MatchValue(_compile_expr(node.value, macros))
    if isinstance(node, MatchAs):
        return ast.MatchAs(name=node.name)
    if isinstance(node, MatchSequence):
        return ast.MatchSequence(patterns=[_compile_pattern(pattern, macros) for pattern in node.patterns])
    if isinstance(node, MatchOr):
        return ast.MatchOr(patterns=[_compile_pattern(pattern, macros) for pattern in node.patterns])
    raise TypeError(f"Unsupported pattern node: {type(node).__name__}")


def _compile_class_setattr(class_name: str, when_fields: List[str]) -> ast.FunctionDef:
    if not when_fields:
        return ast.FunctionDef(
            name="__setattr__",
            args=ast.arguments(
                posonlyargs=[],
                args=[ast.arg(arg="self"), ast.arg(arg="name"), ast.arg(arg="value")],
                kwonlyargs=[],
                kw_defaults=[],
                defaults=[],
            ),
            body=[
                ast.Expr(
                    value=ast.Call(
                        func=ast.Attribute(
                            value=ast.Call(func=ast.Name(id="super", ctx=ast.Load()), args=[], keywords=[]),
                            attr="__setattr__",
                            ctx=ast.Load(),
                        ),
                        args=[ast.Name(id="name", ctx=ast.Load()), ast.Name(id="value", ctx=ast.Load())],
                        keywords=[],
                    )
                )
            ],
            decorator_list=[],
            returns=None,
        )
    field_names: List[ast.expr] = [ast.Constant(value=field) for field in when_fields]
    hook_name_expr = ast.BinOp(
        left=ast.Constant(value=f"{class_name}."),
        op=ast.Add(),
        right=ast.Name(id="name", ctx=ast.Load()),
    )
    return ast.FunctionDef(
        name="__setattr__",
        args=ast.arguments(
            posonlyargs=[],
            args=[ast.arg(arg="self"), ast.arg(arg="name"), ast.arg(arg="value")],
            kwonlyargs=[],
            kw_defaults=[],
            defaults=[],
        ),
        body=[
            ast.If(
                test=ast.Compare(
                    left=ast.Name(id="name", ctx=ast.Load()),
                    ops=[ast.In()],
                    comparators=[ast.Tuple(elts=field_names, ctx=ast.Load())],
                ),
                body=[
                    ast.Assign(
                        targets=[ast.Name(id="value", ctx=ast.Store())],
                        value=ast.Call(
                            func=ast.Name(id="_pypp_apply_hook", ctx=ast.Load()),
                            args=[hook_name_expr, ast.Name(id="value", ctx=ast.Load())],
                            keywords=[],
                        ),
                    )
                ],
                orelse=[],
            ),
            ast.Expr(
                value=ast.Call(
                    func=ast.Attribute(
                        value=ast.Call(func=ast.Name(id="super", ctx=ast.Load()), args=[], keywords=[]),
                        attr="__setattr__",
                        ctx=ast.Load(),
                    ),
                    args=[ast.Name(id="name", ctx=ast.Load()), ast.Name(id="value", ctx=ast.Load())],
                    keywords=[],
                )
            ),
        ],
        decorator_list=[],
        returns=None,
    )


def __compile_method_name(name: str, class_name: str | None) -> str:
    if class_name is not None and name == class_name:
        return "__init__"
    return name


def _compile_use_helpers() -> List[ast.stmt]:
    helper_code = textwrap.dedent(
        """
        def __pypp_import_file(path, alias):
            from pathlib import Path
            from importlib import util
            import types

            path = Path(path)
            if not path.is_absolute():
                base = Path(globals().get("__file__", Path.cwd())).resolve().parent
                path = base / path

            spec = util.spec_from_file_location(alias, str(path))
            if spec is None or spec.loader is None:
                raise ImportError(f"Cannot import module from {path}")
            module = types.ModuleType(alias)
            module.__file__ = str(path)
            module.__name__ = alias
            spec.loader.exec_module(module)
            return module

        def __pypp_import_pypp_file(path, alias):
            from pathlib import Path
            import types
            from pyplusplus.parser import parse_pyplusplus_file
            from pyplusplus.compiler import compile_pyplusplus_to_python_ast

            path = Path(path)
            if not path.is_absolute():
                base = Path(globals().get("__file__", Path.cwd())).resolve().parent
                path = base / path

            source_ast = parse_pyplusplus_file(path)
            module = types.ModuleType(alias)
            module.__file__ = str(path)
            module.__name__ = alias
            exec(compile(compile_pyplusplus_to_python_ast(source_ast, source_path=path), str(path), "exec", dont_inherit=True), module.__dict__, module.__dict__)
            return module
        """
    )
    module = ast.parse(helper_code)
    return [stmt for stmt in module.body]


def _get_default_alias(source: str) -> str:
    import os

    return os.path.splitext(os.path.basename(source))[0]


def _compile_augop(op: str) -> ast.operator:
    augop_map: Dict[str, type[ast.operator]] = {
        "+=": ast.Add,
        "-=": ast.Sub,
        "*=": ast.Mult,
        "/=": ast.Div,
        "%=": ast.Mod,
        "//=": ast.FloorDiv,
        "**=": ast.Pow,
        "<<=": ast.LShift,
        ">>=": ast.RShift,
        "&=": ast.BitAnd,
        "|=": ast.BitOr,
        "^=": ast.BitXor,
    }
    return augop_map[op]()
