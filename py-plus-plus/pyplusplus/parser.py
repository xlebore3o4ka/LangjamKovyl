from __future__ import annotations

from pyplusplus.pypp_errors import E01Error

"""Parser for the Py++ language.

This module transforms a stream of tokens produced by the lexer into the
Py++ AST defined in `pyplusplus.ast_nodes`. The parser supports statements,
expressions, function definitions, classes, macros, match patterns, and many
Python-like constructs while maintaining Py++ syntax extensions.
"""

from pathlib import Path

from .ast_nodes import (
    Arg,
    AnnAssign,
    ArgKind,
    Assign,
    AssignTarget,
    Assert,
    AugAssign,
    Attribute,
    Await,
    BinOp,
    BlockExpr,
    BlockStmt,
    BodyStmt,
    BoolOp,
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
    FromUseName,
    FunctionDef,
    GeneratorExp,
    Global,
    If,
    IfExpr,
    Lambda,
    List,
    ListComp,
    ListExpr,
    MacroCall,
    MacroDef,
    MacroParam,
    MacroParamKind,
    Match,
    MatchAs,
    MatchCase,
    MatchSequence,
    MatchValue,
    Module,
    Name,
    Nonlocal,
    ExceptHandler,
    Pass,
    Pattern,
    Raise,
    Return,
    FString,
    Slice,
    Stmt,
    Subscript,
    SetComp,
    SetExpr,
    TupleExpr,
    Try,
    UnaryOp,
    Use,
    UseSource,
    When,
    While,
    With,
    Yield,
    Break,
)
from .lexer import Token, TokenKind, TokenValue, tokenize


def parse_pyplusplus(source: str) -> Module:
    """
    Parse Py++ source code into the internal AST representation.
    Args:
        source (str): The Py++ source code to parse.

    Returns:
        Module: The parsed Py++ AST module.
    Raises:
        SyntaxError: If the source code contains syntax errors.
    """
    tokens: List[Token] = tokenize(source)
    parser: Parser = Parser(tokens)
    return parser.parse_module()


def parse_pyplusplus_file(path: Path | str) -> Module:
    """
    Read a source file and parse it as Py++ code.

    Args:
        path (Path | str): The path to the Py++ source file.

    Returns:
        Module: The parsed Py++ AST module.

    Raises:
        SyntaxError: If the source code contains syntax errors.
    """
    text: str = Path(path).read_text(encoding="utf-8")
    return parse_pyplusplus(text)


class Parser:
    """
    Recursive descent parser tracking a token stream position.
    """

    def __init__(self, tokens: List[Token]):
        self.tokens: List[Token] = tokens
        self.pos: int = 0

    @property
    def token(self):
        """
        Return the current token without advancing.
        Returns:
            Token: The current token.
        """
        return self.tokens[self.pos]

    def advance(self):
        """
        Advance to the next token and return the current one.

        Returns:
            Token: The current token after advancing.
        """
        if self.pos < len(self.tokens) - 1:
            self.pos += 1
        return self.token

    def expect(self, kind: TokenKind, value: TokenValue =None) -> Token:
        """
        Expect the current token to match the given kind and optional value, advancing if it does.
        Raises a SyntaxError if the expectation is not met.
        Args:
            kind (TokenKind): The expected token kind.
            value (TokenValue, optional): The expected token value. Defaults to None.
            
        Returns:
            Token: The current token if it matches the expectation.
        Raises:
            SyntaxError: If the current token does not match the expected kind and value.
        """
        token = self.token
        if token.kind != kind or (value is not None and token.value != value):
            raise SyntaxError(
                f"Expected {kind} {value!r} at line {token.line}, col {token.column}, got {token.kind} {token.value!r}"
            )
        self.advance()
        return token

    def match(self, kind: TokenKind, value: TokenValue =None) -> bool:
        """
        Check if the current token matches the given kind and optional value without advancing.
        Args:
            kind (TokenKind): The expected token kind.
            value (TokenValue, optional): The expected token value. Defaults to None.

        Returns:
            bool: True if the current token matches the given kind and value, False otherwise.
        """
        token = self.token
        if token.kind != kind:
            return False
        return value is None or token.value == value

    def parse_module(self) -> Module:
        """
        Parse the entire input into a top-level Module node.

        Returns:
            Module: The top-level module containing all statements.
        Raises:
            SyntaxError: If the source code contains syntax errors.
        """
        body: List[Stmt] = []
        while not self.match(TokenKind.EOF):
            if self.match(TokenKind.SEMICOLON):
                self.advance()
                continue
            body.append(self.parse_statement())
        return Module(body)

    def parse_statement(self) -> Stmt:
        """
        Parse a single statement, handling decorators, async functions, and various statement types.
        
        Returns:
            Stmt: The parsed statement node.
        Raises:
            SyntaxError: If the statement is invalid or unexpected.
        """
        
        # Handle decorators
        decorators: List[Name | Attribute | Call] = []
        while self.match(TokenKind.AT):
            decorators.append(self.parse_decorator())
            if self.match(TokenKind.SEMICOLON):
                self.advance()

        # Handle async functions
        is_async = False
        if self.match(TokenKind.KEYWORD, "async"):
            is_async = True
            self.advance()

        # Handle various statement types
        if self.match(TokenKind.OP, "++") or self.match(TokenKind.OP, "--"):
            op = self.token.value
            self.advance()
            target = self.parse_assignment_target()
            aug_op = "+=" if op == "++" else "-="
            stmt = AugAssign(target=target, op=aug_op, value=Constant(value=1))
        elif self.match(TokenKind.KEYWORD, "if"):
            stmt = self.parse_if()
        elif self.match(TokenKind.KEYWORD, "for"):
            stmt = self.parse_for()
        elif self.match(TokenKind.KEYWORD, "while"):
            stmt = self.parse_while()
        elif self.match(TokenKind.KEYWORD, "func"):
            stmt = self.parse_function_def(is_async=is_async)
        elif self.match(TokenKind.KEYWORD, "when"):
            stmt = self.parse_when()
        elif self.match(TokenKind.KEYWORD, "class"):
            stmt = self.parse_class()
        elif self.match(TokenKind.KEYWORD, "macro"):
            stmt = self.parse_macro()
        elif self.match(TokenKind.KEYWORD, "return"):
            stmt = self.parse_return()
        elif self.match(TokenKind.KEYWORD, "pass"):
            stmt = self.parse_pass()
        elif self.match(TokenKind.KEYWORD, "use"):
            stmt = self.parse_use()
        elif self.match(TokenKind.KEYWORD, "break"):
            stmt = self.parse_break()
        elif self.match(TokenKind.KEYWORD, "continue"):
            stmt = self.parse_continue()
        elif self.match(TokenKind.KEYWORD, "with"):
            stmt = self.parse_with()
        elif self.match(TokenKind.KEYWORD, "from"):
            stmt = self.parse_from_use()
        elif self.match(TokenKind.KEYWORD, "match"):
            stmt = self.parse_match()
        elif self.match(TokenKind.KEYWORD, "assert"):
            stmt = self.parse_assert()
        elif self.match(TokenKind.KEYWORD, "raise"):
            stmt = self.parse_raise()
        elif self.match(TokenKind.KEYWORD, "try"):
            stmt = self.parse_try()
        elif self.match(TokenKind.KEYWORD, "del"):
            stmt = self.parse_del()
        elif self.match(TokenKind.KEYWORD, "global"):
            stmt = self.parse_global()
        elif self.match(TokenKind.KEYWORD, "nonlocal"):
            stmt = self.parse_nonlocal()
        elif self._peek_assignable():
            stmt = self.parse_assign()
        else:
            expr = self.parse_expression()
            if self.match(TokenKind.OP, "++") or self.match(TokenKind.OP, "--"):
                op = self.token.value
                self.advance()
                if not isinstance(expr, (Name, Attribute, Subscript)):
                    raise SyntaxError(
                        f"Invalid target for {op} at line {self.token.line}, col {self.token.column}"
                    )
                aug_op = "+=" if op == "++" else "-="
                stmt = AugAssign(target=expr, op=aug_op, value=Constant(value=1))
            else:
                stmt = ExprStmt(expr)

        # Handle decorators for functions and classes
        if decorators:
            if isinstance(stmt, FunctionDef) or isinstance(stmt, ClassDef):
                stmt.decorators = decorators
            else:
                raise SyntaxError("Decorators can only be applied to functions or classes")
        return stmt

    def parse_decorator(self) -> Name | Attribute | Call:
        self.expect(TokenKind.AT)
        decorator = self.parse_expression()
        if not isinstance(decorator, (Name, Attribute, Call)):
            raise SyntaxError(
                f"Invalid decorator expression at line {self.token.line}, col {self.token.column}"
            )
        return decorator

    def parse_assign(self) -> Stmt:
        """
        Parse an assignment statement. 
        Checks for annotated assignments, simple assignments, and augmented assignments.

        Returns:
            Stmt: The parsed assignment statement node.
        
        Raises:
            SyntaxError: If the assignment statement is invalid or unexpected.
        """
        target: AssignTarget = self.parse_assignment_target()
        
        # Handle optional type annotation
        annotation = None
        if self.match(TokenKind.COLON):
            self.advance()
            annotation = self.parse_type_expression()
        
        # Handle assignment operators
        if self.match(TokenKind.OP):
            op: TokenValue = self.expect(TokenKind.OP).value
            if not isinstance(op, str):
                raise SyntaxError(f"Expected assignment operator at line {self.token.line}, col {self.token.column}, got {op!r}")
            value = self.parse_expression()
            
            # Handle simple assignment
            if op == "=":
                if annotation is not None:
                    return AnnAssign(target=target, annotation=annotation, value=value, simple=isinstance(target, Name))
                return Assign(target=target, value=value)
            
            # Handle augmented assignment operators
            if annotation is not None:
                raise SyntaxError("Annotated assignment cannot use augmented assignment")
            return AugAssign(target=target, op=op, value=value)
        
        # Handle annotated assignment without a value
        if annotation is not None:
            return AnnAssign(target=target, annotation=annotation, value=None, simple=isinstance(target, Name))
        raise SyntaxError(
            f"Expected assignment operator at line {self.token.line}, col {self.token.column}, got {self.token.kind} {self.token.value!r}"
        )

    def parse_assignment_target(self) -> AssignTarget:
        """
        Parse an assignment target, which can be a name, attribute, subscript, or tuple/list of targets.
        
        Returns:
            AssignTarget: The parsed assignment target node.
            
        Raises:
            SyntaxError: If the assignment target is invalid or unexpected.
        """
        def parse_single_target() -> AssignTarget:
            """
            Parse a single assignment target, which can be a name, attribute, or subscript.

            Returns:
                AssignTarget: The parsed assignment target node.

            Raises:
                SyntaxError: If the assignment target is invalid or unexpected.
            """
            
            # Handle simple name targets
            if not self.match(TokenKind.NAME):
                raise SyntaxError(
                    f"Expected assignment target at line {self.token.line}, col {self.token.column}, got {self.token.kind} {self.token.value!r}"
                )
            if not isinstance(self.token.value, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            node = Name(id=self.token.value)
            self.advance()
            while True:
                # Handle attribute access and subscript access for assignment targets
                if self.match(TokenKind.DOT):
                    self.advance()
                    attr = self.expect(TokenKind.NAME).value
                    if not isinstance(attr, str):
                        raise E01Error(
                            token_type=self.token.kind.name,
                            actual_type=type(self.token.value).__name__,
                            line_number=self.token.line
                        )
                    node = Attribute(value=node, attr=attr)
                    continue
                # Handle subscript access for assignment targets
                if self.match(TokenKind.LBRACKET):
                    self.advance()
                    slice_expr = self.parse_slice()
                    self.expect(TokenKind.RBRACKET)
                    node = Subscript(value=node, slice=slice_expr)
                    continue
                break
            return node

        # Handle tuple assignment targets
        if self.match(TokenKind.LPAREN):
            self.advance()
            targets: List[Expr] = []
            if not self.match(TokenKind.RPAREN):
                targets.append(self.parse_assignment_target())
                while self.match(TokenKind.COMMA):
                    self.advance()
                    if self.match(TokenKind.RPAREN):
                        break
                    targets.append(self.parse_assignment_target())
            self.expect(TokenKind.RPAREN)
            return TupleExpr(elements=targets)

        # Handle list assignment targets
        if self.match(TokenKind.LBRACKET):
            self.advance()
            targets: List[Expr] = []
            if not self.match(TokenKind.RBRACKET):
                targets.append(self.parse_assignment_target())
                while self.match(TokenKind.COMMA):
                    self.advance()
                    if self.match(TokenKind.RBRACKET):
                        break
                    targets.append(self.parse_assignment_target())
            self.expect(TokenKind.RBRACKET)
            return ListExpr(elements=targets)
        
        # Handle single assignment targets (name, attribute, subscript)
        target = parse_single_target()
        if self.match(TokenKind.COMMA):
            targets = [target]
            while self.match(TokenKind.COMMA):
                self.advance()
                if self.match(TokenKind.RPAREN):
                    break
                targets.append(self.parse_assignment_target())
            return ListExpr(elements=targets)
        return target

    def _peek_assignable(self) -> bool:
        """
        Peek ahead to check if the current token sequence can form an assignment target.

        Returns:
            bool: True if the current token sequence can form an assignment target, False otherwise.
        """
        pos = self.pos
        if pos >= len(self.tokens) or self.tokens[pos].kind != TokenKind.NAME:
            return False
        pos += 1
        while pos < len(self.tokens) and self.tokens[pos].kind in {TokenKind.DOT, TokenKind.LBRACKET}:
            if self.tokens[pos].kind == TokenKind.DOT:
                pos += 1
                if pos >= len(self.tokens) or self.tokens[pos].kind != TokenKind.NAME:
                    return False
                pos += 1
                continue
            if self.tokens[pos].kind == TokenKind.LBRACKET:
                pos += 1
                depth = 1
                while pos < len(self.tokens) and depth > 0:
                    if self.tokens[pos].kind == TokenKind.LBRACKET:
                        depth += 1
                    elif self.tokens[pos].kind == TokenKind.RBRACKET:
                        depth -= 1
                    pos += 1
                if depth != 0:
                    return False
                continue
        if pos < len(self.tokens) and (
            self.tokens[pos].kind == TokenKind.COLON
            or (
                self.tokens[pos].kind == TokenKind.OP
                and self.tokens[pos].value in {"=", "+=", "-=", "*=", "/=", "%=", "//=", "**=", "<<=", ">>=", "&=", "|=", "^="}
            )
        ):
            return True
        return False

    def parse_body(self) -> BodyStmt:
        """
        Parse a body of statements, which can be a single statement or a block of statements.
        
        Returns:
            BodyStmt: The parsed body, either as a single statement or a block of statements.
            
        Raises:
            SyntaxError: If the body is invalid or unexpected.
        """
        if not self.match(TokenKind.LBRACE):
            stmt = self.parse_statement()
            if self.match(TokenKind.SEMICOLON):
                self.advance()
            return BlockStmt(body=[stmt])
        return self.parse_block()

    def parse_if(self) -> If:
        """
        Parse an if statement, including optional elif and else clauses.
        
        Returns:
            If: The parsed if statement node, including any elif and else clauses.
        
        Raises:
            SyntaxError: If the if statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "if")
        test = self.parse_expression()
        body = self.parse_body()
        current = If(test=test, body=body, orelse=BlockStmt(body=[]))
        root = current
        
        # Handle optional elif clauses
        while self.match(TokenKind.KEYWORD, "elif"):
            self.advance()
            next_test = self.parse_expression()
            next_body = self.parse_body()
            next_if = If(test=next_test, body=next_body, orelse=BlockStmt(body=[]))
            current.orelse = next_if
            current = next_if
        
        # Handle optional else clause
        if self.match(TokenKind.KEYWORD, "else"):
            self.advance()
            else_body = self.parse_body()
            current.orelse = else_body
        return root

    def parse_for_loop_target(self) -> ForLoopTarget:
        """
        Parse a for loop target, which can be a name, tuple, or list of targets.
        
        Returns:
            ForLoopTarget: The parsed for loop target node.
            
        Raises:
            SyntaxError: If the for loop target is invalid or unexpected.
        """
        if self.match(TokenKind.NAME):
            id = self.expect(TokenKind.NAME).value
            if not isinstance(id, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            return Name(id=id)
        if self.match(TokenKind.LPAREN):
            self.advance()
            elements: List[Expr] = []
            while not self.match(TokenKind.RPAREN):
                elements.append(self.parse_for_loop_target())
                if self.match(TokenKind.COMMA):
                    self.advance()
            self.expect(TokenKind.RPAREN)
            return TupleExpr(elements=elements)
        if self.match(TokenKind.LBRACKET):
            self.advance()
            elements: List[Expr] = []
            while not self.match(TokenKind.RBRACKET):
                elements.append(self.parse_for_loop_target())
                if self.match(TokenKind.COMMA):
                    self.advance()
            self.expect(TokenKind.RBRACKET)
            return ListExpr(elements=elements)
        raise SyntaxError("Invalid for loop target")
    
    def parse_for(self) -> For:
        """
        Parse a for loop statement.

        Returns:
            For: The parsed for loop statement node.

        Raises:
            SyntaxError: If the for loop statement is invalid or unexpected.
        """
        
        self.expect(TokenKind.KEYWORD, "for")
        self.expect(TokenKind.LPAREN)
        target = self.parse_for_loop_target()
        self.expect(TokenKind.KEYWORD, "in")
        iterable = self.parse_expression()
        self.expect(TokenKind.RPAREN)
        body = self.parse_body()
        return For(target=target, iter=iterable, body=body)

    def parse_while(self) -> While:
        """
        Parse a while loop statement.
        
        Returns:
            While: The parsed while loop statement node.
        
        Raises:
            SyntaxError: If the while loop statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "while")
        self.expect(TokenKind.LPAREN)
        test = self.parse_expression()
        self.expect(TokenKind.RPAREN)
        body = self.parse_body()
        if self.match(TokenKind.KEYWORD, "else"):
            self.advance()
            orelse = self.parse_body()
            return While(test=test, body=body, orelse=orelse)
        return While(test=test, body=body, orelse=None)

    def parse_function_def(self, is_async: bool = False) -> FunctionDef:
        """
        Parse a function definition statement, including its name, arguments, return type, and body.
        
        Returns:
            FunctionDef: The parsed function definition statement node.

        Raises:
            SyntaxError: If the function definition statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "func")
        name_token = self.expect(TokenKind.NAME)
        if not isinstance(name_token.value, str):
            raise E01Error(
                token_type=name_token.kind.name,
                actual_type=type(name_token.value).__name__,
                line_number=name_token.line
            )
        self.expect(TokenKind.LPAREN)
        args: List[Arg] = []
        saw_kwonly = False
        
        # Handle function arguments, including positional, keyword-only, varargs, and kwargs
        while not self.match(TokenKind.RPAREN):
            # Handle keyword arguments (**kwargs)
            if self.match(TokenKind.OP, "**"):
                self.advance()
                arg_name = self.expect(TokenKind.NAME).value
                if not isinstance(arg_name, str):
                    raise E01Error(
                        token_type=self.token.kind.name,
                        actual_type=type(self.token.value).__name__,
                        line_number=self.token.line
                    )
                annotation = None
                if self.match(TokenKind.COLON):
                    self.advance()
                    annotation = self.parse_expression()
                args.append(Arg(name=arg_name, default=None, annotation=annotation, kind=ArgKind.KWARG))
            
            # Handle variable arguments (*args)
            elif self.match(TokenKind.OP, "*"):
                self.advance()
                if self.match(TokenKind.NAME):
                    arg_name = self.expect(TokenKind.NAME).value
                    if not isinstance(arg_name, str):
                        raise E01Error(
                            token_type=self.token.kind.name,
                            actual_type=type(self.token.value).__name__,
                            line_number=self.token.line
                        )
                    annotation = None
                    if self.match(TokenKind.COLON):
                        self.advance()
                        annotation = self.parse_expression()
                    args.append(Arg(name=arg_name, default=None, annotation=annotation, kind=ArgKind.VARARG))
                    saw_kwonly = True
                else:
                    saw_kwonly = True
            
            # Handle regular arguments (positional or keyword-only)
            else:
                arg_name = self.expect(TokenKind.NAME).value
                if not isinstance(arg_name, str):
                    raise E01Error(
                        token_type=self.token.kind.name,
                        actual_type=type(self.token.value).__name__,
                        line_number=self.token.line
                    )
                annotation = None
                if self.match(TokenKind.COLON):
                    self.advance()
                    annotation = self.parse_expression()
                default = None
                if self.match(TokenKind.OP, "="):
                    self.advance()
                    default = self.parse_expression()
                kind = ArgKind.KEYWORD_ONLY if saw_kwonly else ArgKind.POSITIONAL
                args.append(Arg(name=arg_name, default=default, annotation=annotation, kind=kind))
            if self.match(TokenKind.COMMA):
                self.advance()
                continue
            break
        self.expect(TokenKind.RPAREN)
        
        # Handle optional return type annotation
        return_type = None
        if self.match(TokenKind.OP, "->"):
            self.advance()
            return_type = self.parse_type_expression()
        body = self.parse_body()
        return FunctionDef(
            name=name_token.value,
            args=args,
            body=body,
            decorators=[],
            is_async=is_async,
            return_type=return_type,
        )

    def parse_when(self) -> When:
        """
        Parse a 'when' statement, which is a conditional construct in Py++.
        
        Returns:
            When: The parsed 'when' statement node.
            
        Raises:
            SyntaxError: If the 'when' statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "when")
        var = self.expect(TokenKind.NAME).value
        if not isinstance(var, str):
            raise E01Error(
                token_type=self.token.kind.name,
                actual_type=type(self.token.value).__name__,
                line_number=self.token.line
            )
        self.expect(TokenKind.LPAREN)
        param = self.expect(TokenKind.NAME).value
        if not isinstance(param, str):
            raise E01Error(
                token_type=self.token.kind.name,
                actual_type=type(self.token.value).__name__,
                line_number=self.token.line
            )
        self.expect(TokenKind.RPAREN)
        body = self.parse_body()
        return When(var=var, param=param, body=body)

    def parse_class(self) -> ClassDef:
        """
        Parse a class definition statement, including its name, base classes, and body.
        
        Returns:
            ClassDef: The parsed class definition statement node.
            
        Raises:
            SyntaxError: If the class definition statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "class")
        name = self.expect(TokenKind.NAME).value
        if not isinstance(name, str):
            raise E01Error(
                token_type=self.token.kind.name,
                actual_type=type(self.token.value).__name__,
                line_number=self.token.line
            )
        bases: List[Expr] = []
        if self.match(TokenKind.KEYWORD, "extends"):
            self.advance()
            class_name = self.parse_expression()
            bases.append(class_name)
            while self.match(TokenKind.COMMA):
                self.advance()
                class_name = self.parse_expression()
                bases.append(class_name)
        body = self.parse_block()
        return ClassDef(name=name, bases=bases, body=body, decorators=[])

    def str_to_macro_param_kind(self, kind_str: str) -> MacroParamKind:
        """
        Convert a string representation of a macro parameter kind to the corresponding MacroParamKind enum value.
        
        Args:
            kind_str (str): The string representation of the macro parameter kind.
            
        Returns:
            MacroParamKind: The corresponding MacroParamKind enum value.
            
        Raises:
            ValueError: If the provided string does not correspond to a valid MacroParamKind.
        """
        if kind_str == "Expr":
            return MacroParamKind.EXPRESSION
        elif kind_str == "Stmt":
            return MacroParamKind.STATEMENT
        elif kind_str in ("Ident", "Identifier"):
            return MacroParamKind.IDENTIFIER
        elif kind_str == "Code":
            return MacroParamKind.CODE
        else:
            raise ValueError(f"Invalid macro parameter kind: {kind_str}")

    def parse_macro(self) -> MacroDef:
        """
        Parse a macro definition statement, including its name, parameters, and body.
        
        Returns:
            MacroDef: The parsed macro definition statement node.
            
        Raises:
            SyntaxError: If the macro definition statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "macro")
        name = self.expect(TokenKind.NAME).value
        if not isinstance(name, str):
            raise E01Error(
                token_type=self.token.kind.name,
                actual_type=type(self.token.value).__name__,
                line_number=self.token.line
            )
            
        # Handle optional macro parameters enclosed in brackets
        params: List[MacroParam] = []
        if self.match(TokenKind.LBRACKET):
            self.advance()
            while not self.match(TokenKind.RBRACKET):
                param_name = self.expect(TokenKind.NAME).value
                if not isinstance(param_name, str):
                    raise E01Error(
                        token_type=self.token.kind.name,
                        actual_type=type(self.token.value).__name__,
                        line_number=self.token.line
                    )
                param_type: MacroParamKind | None = None
                self.expect(TokenKind.COLON)
                _temp = self.expect(TokenKind.NAME).value
                if not isinstance(_temp, str):
                    raise E01Error(
                        token_type=self.token.kind.name,
                        actual_type=type(self.token.value).__name__,
                        line_number=self.token.line
                    )
                param_type = self.str_to_macro_param_kind(_temp)
                params.append(MacroParam(name=param_name, type=param_type))
                if self.match(TokenKind.COMMA):
                    self.advance()
                    continue
                break
            self.expect(TokenKind.RBRACKET)
        body = self.parse_block()
        return MacroDef(name=name, params=params, body=body)

    def parse_macro_call(self) -> MacroCall:
        """
        Parse a macro call expression, including its name and argument expressions.
        
        Returns:
            MacroCall: The parsed macro call expression node.
            
        Raises:
            SyntaxError: If the macro call expression is invalid or unexpected.
        """
        self.expect(TokenKind.DOLLAR)
        name = self.expect(TokenKind.NAME).value
        if not isinstance(name, str):
            raise E01Error(
                token_type=self.token.kind.name,
                actual_type=type(self.token.value).__name__,
                line_number=self.token.line,
            )
        self.expect(TokenKind.LBRACKET)
        args: List[Expr] = []

        def parse_macro_arg() -> Expr:
            if self.match(TokenKind.LBRACE):
                self.advance()
                body: List[Stmt] = []
                while not self.match(TokenKind.RBRACE):
                    if self.match(TokenKind.SEMICOLON):
                        self.advance()
                        continue
                    body.append(self.parse_statement())
                self.expect(TokenKind.RBRACE)
                return BlockExpr(body=body)
            return self.parse_expression()

        if not self.match(TokenKind.RBRACKET):
            args.append(parse_macro_arg())
            while self.match(TokenKind.COMMA):
                self.advance()
                args.append(parse_macro_arg())
        self.expect(TokenKind.RBRACKET)
        return MacroCall(name=name, args=args)

    def parse_assert(self) -> Assert:
        """
        Parse an assert statement, which checks a condition and optionally provides a message.
        
        Returns:
            Assert: The parsed assert statement node.
            
        Raises:
            SyntaxError: If the assert statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "assert")
        test = self.parse_expression()
        msg = None
        if self.match(TokenKind.COMMA):
            self.advance()
            msg = self.parse_expression()
        return Assert(test=test, msg=msg)

    def parse_raise(self) -> Raise:
        """
        Parse a raise statement, which raises an exception.
        
        Returns:
            Raise: The parsed raise statement node.
            
        Raises:
            SyntaxError: If the raise statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "raise")
        
        # Handle the case where no exception is specified (re-raise)
        if self.match(TokenKind.SEMICOLON) or self.match(TokenKind.RBRACE):
            return Raise(exc=None)
        exc = self.parse_expression()
        return Raise(exc=exc)

    def parse_try(self) -> Try:
        """
        Parse a try statement, which includes a body, exception handlers, and optional else and finally clauses.
        
        Returns:
            Try: The parsed try statement node.
            
        Raises:
            SyntaxError: If the try statement is invalid or unexpected.
        """
        
        self.expect(TokenKind.KEYWORD, "try")
        body = self.parse_body()
        handlers: List[ExceptHandler] = []
        orelse: BodyStmt | None = None
        finalbody: BodyStmt | None = None
        while self.match(TokenKind.KEYWORD, "except"):
            handlers.append(self.parse_except_handler())
        if self.match(TokenKind.KEYWORD, "else"):
            self.advance()
            orelse = self.parse_body()
        if self.match(TokenKind.KEYWORD, "finally"):
            self.advance()
            finalbody = self.parse_body()
        return Try(body=body, handlers=handlers, orelse=orelse, finalbody=finalbody)

    def parse_except_handler(self) -> ExceptHandler:
        """
        Parse an except handler, which includes the exception type, optional name, and body.
        
        Returns:
            ExceptHandler: The parsed except handler node.
            
        Raises:
            SyntaxError: If the except handler is invalid or unexpected.
        """
        
        self.expect(TokenKind.KEYWORD, "except")
        type_expr = None
        name = None
        if not self.match(TokenKind.LBRACE) and not self.match(TokenKind.KEYWORD, "except") and not self.match(TokenKind.KEYWORD, "else") and not self.match(TokenKind.KEYWORD, "finally"):
            type_expr = self.parse_expression()
            if self.match(TokenKind.KEYWORD, "as"):
                self.advance()
                name = self.expect(TokenKind.NAME).value
        if name is not None and not isinstance(name, str):
            raise E01Error(
                token_type=self.token.kind.name,
                actual_type=type(self.token.value).__name__,
                line_number=self.token.line
            )
        body = self.parse_body()
        return ExceptHandler(type=type_expr, name=name, body=body)

    def parse_del(self) -> Del:
        """
        Parse a delete statement, which removes one or more targets.
        
        Returns:
            Del: The parsed delete statement node.
            
        Raises:
            SyntaxError: If the delete statement is invalid or unexpected.
        """
        
        self.expect(TokenKind.KEYWORD, "del")
        targets = [self.parse_assignment_target()]
        while self.match(TokenKind.COMMA):
            self.advance()
            targets.append(self.parse_assignment_target())
        return Del(targets=targets)

    def parse_global(self) -> Global:
        """
        Parse a global statement, which declares names as global within a function.
        
        Returns:
            Global: The parsed global statement node.
            
        Raises:
            SyntaxError: If the global statement is invalid or unexpected.
        """
                
        self.expect(TokenKind.KEYWORD, "global")
        name = self.expect(TokenKind.NAME).value
        if not isinstance(name, str):
            raise E01Error(
                token_type=self.token.kind.name,
                actual_type=type(self.token.value).__name__,
                line_number=self.token.line
            )
        names: List[str] = [name]
        while self.match(TokenKind.COMMA):
            self.advance()
            name = self.expect(TokenKind.NAME).value
            if not isinstance(name, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            names.append(name)
        return Global(names=names)

    def parse_nonlocal(self) -> Nonlocal:
        """
        Parse a nonlocal statement, which declares names as nonlocal within a nested function.
        
        Returns:
            Nonlocal: The parsed nonlocal statement node.
            
        Raises:
            SyntaxError: If the nonlocal statement is invalid or unexpected.
        """
        
        self.expect(TokenKind.KEYWORD, "nonlocal")
        name = self.expect(TokenKind.NAME).value
        if not isinstance(name, str):
            raise E01Error(
                token_type=self.token.kind.name,
                actual_type=type(self.token.value).__name__,
                line_number=self.token.line
            )
        names: List[str] = [name]
        while self.match(TokenKind.COMMA):
            self.advance()
            name = self.expect(TokenKind.NAME).value
            if not isinstance(name, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            names.append(name)
        return Nonlocal(names=names)

    def parse_return(self) -> Return:
        """
        Parse a return statement, which may include an optional return value.
        
        Returns:
            Return: The parsed return statement node.
            
        Raises:
            SyntaxError: If the return statement is invalid or unexpected.
        """
        
        self.expect(TokenKind.KEYWORD, "return")
        if self.match(TokenKind.SEMICOLON) or self.match(TokenKind.RBRACE):
            return Return(value=None)
        value = self.parse_expression()
        return Return(value=value)

    def parse_pass(self) -> Pass:
        """
        Parse a pass statement, which does nothing.
        
        Returns:
            Pass: The parsed pass statement node.
            
        Raises:
            SyntaxError: If the pass statement is invalid or unexpected.
        """
        
        self.expect(TokenKind.KEYWORD, "pass")
        return Pass()

    def parse_use(self) -> Use:
        """
        Parse a use statement, which imports a module or specific names from a module.
        
        Returns:
            Use: The parsed use statement node.
            
        Raises:
            SyntaxError: If the use statement is invalid or unexpected.
        """
        
        self.expect(TokenKind.KEYWORD, "use")
        
        # Handle the case where the source is a string literal (e.g., use "module")
        if self.match(TokenKind.STRING):
            if not isinstance(self.token.value, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            source = self.token.value
            self.advance()
            is_string = True
        
        # Handle the case where the source is a name or attribute (e.g., use module or use module.submodule)
        else:
            is_string = False
            id = self.expect(TokenKind.NAME).value
            if not isinstance(id, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            source = Name(id=id)
            
            # Handle attribute access for use statements (e.g., use module.submodule)
            while True:
                if self.match(TokenKind.DOT):
                    self.advance()
                    attr = self.expect(TokenKind.NAME).value
                    if not isinstance(attr, str):
                        raise E01Error(
                            token_type=self.token.kind.name,
                            actual_type=type(self.token.value).__name__,
                            line_number=self.token.line
                        )
                    source = Attribute(value=source, attr=attr)
                    continue
                break
        alias = None
        if self.match(TokenKind.KEYWORD, "as"):
            self.advance()
            alias = self.expect(TokenKind.NAME).value
            if not isinstance(alias, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
        return Use(source=UseSource(path=source, is_string=is_string), alias=alias)

    def parse_break(self) -> Break:
        """
        Parse a break statement, which exits the nearest enclosing loop.
        
        Returns:
            Break: The parsed break statement node.
            
        Raises:
            SyntaxError: If the break statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "break")
        return Break()

    def parse_continue(self) -> Continue:
        """
        Parse a continue statement, which skips the rest of the current loop iteration and continues with the
        
        Returns:
            Continue: The parsed continue statement node.
            
        Raises:
            SyntaxError: If the continue statement is invalid or unexpected.
        """
        
        self.expect(TokenKind.KEYWORD, "continue")
        return Continue()

    def parse_with(self) -> With:
        """
        Parse a with statement, which manages context for a block of code.
        
        Returns:
            With: The parsed with statement node.
            
        Raises:
            SyntaxError: If the with statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "with")
        self.expect(TokenKind.LPAREN)
        context_expr = self.parse_expression()
        alias = None
        if self.match(TokenKind.KEYWORD, "as"):
            self.advance()
            alias = self.expect(TokenKind.NAME).value
            if not isinstance(alias, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
        self.expect(TokenKind.RPAREN)
        body = self.parse_body()
        return With(context_expr=context_expr, alias=alias, body=body)

    def parse_from_use(self) -> FromUse:
        """
        Parse a from use statement, which imports specific names from a module or package.

        Returns:
            FromUse: The parsed from use statement node.

        Raises:
            SyntaxError: If the from use statement is invalid or unexpected.
        """
        
        self.expect(TokenKind.KEYWORD, "from")
        if self.match(TokenKind.STRING):
            if not isinstance(self.token.value, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            source = UseSource(path=self.token.value, is_string=True)
            self.advance()
        else:
            id = self.expect(TokenKind.NAME).value
            if not isinstance(id, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            path = Name(id=id)
            while True:
                if self.match(TokenKind.DOT):
                    self.advance()
                    attr = self.expect(TokenKind.NAME).value
                    if not isinstance(attr, str):
                        raise E01Error(
                            token_type=self.token.kind.name,
                            actual_type=type(self.token.value).__name__,
                            line_number=self.token.line
                        )
                    path = Attribute(value=path, attr=attr)
                    continue
                break
            source = UseSource(path=path, is_string=False)
        self.expect(TokenKind.KEYWORD, "use")
        names: List[FromUseName] = []
        while True:
            if self.match(TokenKind.STRING):
                name = self.token.value
                is_string = True
                self.advance()
            else:
                name = self.expect(TokenKind.NAME).value
                is_string = False
            alias = None
            if self.match(TokenKind.KEYWORD, "as"):
                self.advance()
                alias = self.expect(TokenKind.NAME).value
            if not isinstance(name, str):
                raise SyntaxError(f"Expected name or string literal in from use statement at line {self.token.line}, col {self.token.column}, got {name!r}")
            if alias is not None and not isinstance(alias, str):
                raise SyntaxError(f"Expected alias name in from use statement at line {self.token.line}, col {self.token.column}, got {alias!r}")
            names.append(FromUseName(name=name, is_string=is_string, alias=alias))
            if not self.match(TokenKind.COMMA):
                break
            self.advance()
        return FromUse(source=source, names=names)

    def parse_match(self) -> Match:
        """
        Parse a match statement, which allows structural pattern matching.

        Returns:
            Match: The parsed match statement node.

        Raises:
            SyntaxError: If the match statement is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "match")
        self.expect(TokenKind.LPAREN)
        subject = self.parse_expression()
        self.expect(TokenKind.RPAREN)
        self.expect(TokenKind.LBRACE)
        cases: List[MatchCase] = []
        while self.match(TokenKind.KEYWORD, "case"):
            self.advance()
            pattern = self.parse_pattern()
            guard = None
            if self.match(TokenKind.KEYWORD, "if"):
                self.advance()
                guard = self.parse_expression()
            self.expect(TokenKind.COLON)
            body = self.parse_body()
            cases.append(MatchCase(pattern=pattern, guard=guard, body=body))
        self.expect(TokenKind.RBRACE)
        return Match(subject=subject, cases=cases)

    def parse_lambda(self) -> Lambda:
        """
        Parse a lambda expression, which is an anonymous function with optional arguments and a return type.
        
        Returns:
            Lambda: The parsed lambda expression node.
            
        Raises:
            SyntaxError: If the lambda expression is invalid or unexpected.
        """
        self.expect(TokenKind.KEYWORD, "func")
        self.expect(TokenKind.LPAREN)
        args: List[Arg] = []
        saw_kwonly = False
        
        # Handle lambda arguments, including positional, keyword-only, varargs, and kwargs
        while not self.match(TokenKind.RPAREN):
            # Handle keyword arguments (**kwargs)
            if self.match(TokenKind.OP, "**"):
                self.advance()
                arg_name = self.expect(TokenKind.NAME).value
                if not isinstance(arg_name, str):
                    raise E01Error(
                        token_type=self.token.kind.name,
                        actual_type=type(self.token.value).__name__,
                        line_number=self.token.line
                    )
                annotation = None
                if self.match(TokenKind.COLON):
                    self.advance()
                    annotation = self.parse_expression()
                args.append(Arg(name=arg_name, default=None, annotation=annotation, kind=ArgKind.KWARG))
            
            # Handle variable arguments (*args)
            elif self.match(TokenKind.OP, "*"):
                self.advance()
                if self.match(TokenKind.NAME):
                    arg_name = self.expect(TokenKind.NAME).value
                    if not isinstance(arg_name, str):
                        raise E01Error(
                            token_type=self.token.kind.name,
                            actual_type=type(self.token.value).__name__,
                            line_number=self.token.line
                        )
                    annotation = None
                    if self.match(TokenKind.COLON):
                        self.advance()
                        annotation = self.parse_expression()
                    args.append(Arg(name=arg_name, default=None, annotation=annotation, kind=ArgKind.VARARG))
                    saw_kwonly = True
                else:
                    saw_kwonly = True
            
            # Handle regular arguments (positional or keyword-only)
            else:
                arg_name = self.expect(TokenKind.NAME).value
                if not isinstance(arg_name, str):
                    raise E01Error(
                        token_type=self.token.kind.name,
                        actual_type=type(self.token.value).__name__,
                        line_number=self.token.line
                    )
                annotation = None
                if self.match(TokenKind.COLON):
                    self.advance()
                    annotation = self.parse_expression()
                default = None
                if self.match(TokenKind.OP, "="):
                    self.advance()
                    default = self.parse_expression()
                kind = ArgKind.KEYWORD_ONLY if saw_kwonly else ArgKind.POSITIONAL
                args.append(Arg(name=arg_name, default=default, annotation=annotation, kind=kind))
            if self.match(TokenKind.COMMA):
                self.advance()
                continue
            break
        self.expect(TokenKind.RPAREN)
        
        # Handle optional return type annotation
        return_type = None
        if self.match(TokenKind.OP, "->"):
            self.advance()
            return_type = self.parse_type_expression()
        
        self.expect(TokenKind.LBRACE)
        expr = self.parse_expression()
        self.expect(TokenKind.RBRACE)
        return Lambda(args=args, return_type=return_type, expr=expr)

    def parse_brace_literal(self) -> DictExpr | SetExpr | DictComp | SetComp:
        """
        Parse a brace literal, which can represent a dictionary, set, or comprehension.
        
        Returns:
            DictExpr | SetExpr | DictComp | SetComp: The parsed brace literal expression node.
            
        Raises:
            SyntaxError: If the brace literal is invalid or unexpected.
        """
        self.expect(TokenKind.LBRACE)
        if self.match(TokenKind.RBRACE):
            self.advance()
            return DictExpr(keys=[], values=[])
        first = self.parse_expression()
        if self.match(TokenKind.COLON):
            self.advance()
            value = self.parse_expression()
            if self.match(TokenKind.KEYWORD, "for"):
                generators = self.parse_comprehension_generators()
                self.expect(TokenKind.RBRACE)
                return DictComp(key=first, value=value, generators=generators)
            keys = [first]
            values = [value]
            while self.match(TokenKind.COMMA):
                self.advance()
                if self.match(TokenKind.RBRACE):
                    break
                keys.append(self.parse_expression())
                self.expect(TokenKind.COLON)
                values.append(self.parse_expression())
            self.expect(TokenKind.RBRACE)
            return DictExpr(keys=keys, values=values)
        if self.match(TokenKind.KEYWORD, "for"):
            generators = self.parse_comprehension_generators()
            self.expect(TokenKind.RBRACE)
            return SetComp(elt=first, generators=generators)
        elements = [first]
        while self.match(TokenKind.COMMA):
            self.advance()
            if self.match(TokenKind.RBRACE):
                break
            elements.append(self.parse_expression())
        self.expect(TokenKind.RBRACE)
        return SetExpr(elements=elements)

    def parse_slice(self) -> Expr:
        """
        Parse a slice or tuple subscript expression.
        
        Returns:
            Expr: The parsed slice expression node or tuple subscript node.
        
        Raises:
            SyntaxError: If the slice expression is invalid or unexpected.
        """
        lower: Expr | None = None
        upper: Expr | None = None
        step: Expr | None = None
        if self.match(TokenKind.COLON):
            self.advance()
        else:
            lower = self.parse_expression()
        if self.match(TokenKind.COLON):
            self.advance()
            if not self.match(TokenKind.RBRACKET):
                upper = self.parse_expression()
            if self.match(TokenKind.COLON):
                self.advance()
                if not self.match(TokenKind.RBRACKET):
                    step = self.parse_expression()
            return Slice(lower=lower, upper=upper, step=step)
        if self.match(TokenKind.COMMA):
            elements: List[Expr] = [lower] if lower is not None else []
            while self.match(TokenKind.COMMA):
                self.advance()
                if self.match(TokenKind.RBRACKET):
                    break
                elements.append(self.parse_expression())
            return TupleExpr(elements=elements)
        if lower is None:
            raise SyntaxError(
                f"Expected slice or subscript expression at line {self.token.line}, col {self.token.column}"
            )
        return lower

    def parse_comprehension_generators(self) -> List[Comprehension]:
        """
        Parse comprehension generators, which are used in list, set, and dictionary comprehensions.
        
        Returns:
            List[Comprehension]: A list of parsed comprehension generator nodes.
            
        Raises:
            SyntaxError: If the comprehension generators are invalid or unexpected.
        """
        
        generators: List[Comprehension] = []
        while self.match(TokenKind.KEYWORD, "for"):
            self.advance()
            target = self.parse_assignment_target()
            self.expect(TokenKind.KEYWORD, "in")
            iterator = self.parse_expression()
            ifs: List[Expr] = []
            while self.match(TokenKind.KEYWORD, "if"):
                self.advance()
                ifs.append(self.parse_expression())
            generators.append(Comprehension(target=target, iter=iterator, ifs=ifs))
        return generators

    def parse_pattern(self) -> Pattern:
        """
        Parse a pattern used in match statements.
        
        Returns:
            Pattern: The parsed pattern node.
            
        Raises:
            SyntaxError: If the pattern is invalid or unexpected.
        """
        
        if self.match(TokenKind.NAME):
            name = self.token.value
            if not isinstance(name, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            self.advance()
            if name == "_":
                return MatchAs(name=None)
            return MatchAs(name=name)
        if self.match(TokenKind.KEYWORD) and self.token.value in {"True", "False", "None"}:
            raw = self.token.value
            self.advance()
            return MatchValue(Constant(value=True if raw == "True" else False if raw == "False" else None))
        if self.match(TokenKind.NUMBER):
            value = Constant(value=self.token.value)
            self.advance()
            return MatchValue(value)
        if self.match(TokenKind.STRING):
            value = Constant(value=self.token.value)
            self.advance()
            return MatchValue(value)
        if self.match(TokenKind.LBRACKET):
            self.advance()
            patterns: List[Pattern] = []
            if not self.match(TokenKind.RBRACKET):
                patterns.append(self.parse_pattern())
                while self.match(TokenKind.COMMA):
                    self.advance()
                    if self.match(TokenKind.RBRACKET):
                        break
                    patterns.append(self.parse_pattern())
            self.expect(TokenKind.RBRACKET)
            return MatchSequence(patterns=patterns)
        if self.match(TokenKind.LPAREN):
            self.advance()
            patterns: List[Pattern] = []
            if not self.match(TokenKind.RPAREN):
                patterns.append(self.parse_pattern())
                while self.match(TokenKind.COMMA):
                    self.advance()
                    if self.match(TokenKind.RPAREN):
                        break
                    patterns.append(self.parse_pattern())
            self.expect(TokenKind.RPAREN)
            return MatchSequence(patterns=patterns)
        raise SyntaxError(f"Unexpected pattern at line {self.token.line}, col {self.token.column}")

    def parse_block(self) -> BlockStmt:
        """
        Parse a block statement, which is a sequence of statements enclosed in braces.
        
        Returns:
            BlockStmt: The parsed block statement node.
            
        Raises:
            SyntaxError: If the block statement is invalid or unexpected.
        """
        
        self.expect(TokenKind.LBRACE)
        body: List[Stmt] = []
        while not self.match(TokenKind.RBRACE):
            if self.match(TokenKind.SEMICOLON):
                self.advance()
                continue
            body.append(self.parse_statement())
        self.expect(TokenKind.RBRACE)
        return BlockStmt(body=body)

    def parse_expression(self) -> Expr:
        """
        Parse an expression, which can include logical, comparison, and arithmetic operations.
        
        Returns:
            Expr: The parsed expression node.
            
        Raises:
            SyntaxError: If the expression is invalid or unexpected.
        """
        
        return self.parse_or()

    def parse_type_expression(self) -> Expr:
        """
        Parse a type annotation expression.
        
        Returns:
            Expr: The parsed type expression node.
        """
        return self.parse_expression()

    def parse_or(self) -> Expr:
        """
        Parse a logical OR expression, which can include multiple AND expressions combined with 'or' or '||'.
        
        Returns:
            Expr: The parsed logical OR expression node.
            
        Raises:
            SyntaxError: If the logical OR expression is invalid or unexpected.
        """
        
        left = self.parse_and()
        values = [left]
        while self.match(TokenKind.OP, "||") or self.match(TokenKind.KEYWORD, "or"):
            self.advance()
            values.append(self.parse_and())
        if len(values) > 1:
            return BoolOp(op="or", values=values)
        return left

    def parse_and(self) -> Expr:
        """
        Parse a logical AND expression, which can include multiple NOT expressions combined with 'and' or '&&'.
        
        Returns:
            Expr: The parsed logical AND expression node.
            
        Raises:
            SyntaxError: If the logical AND expression is invalid or unexpected.
        """
        
        left = self.parse_not()
        values = [left]
        while self.match(TokenKind.OP, "&&") or self.match(TokenKind.KEYWORD, "and"):
            self.advance()
            values.append(self.parse_not())
        if len(values) > 1:
            return BoolOp(op="and", values=values)
        return left

    def parse_not(self) -> Expr:
        """
        Parse a logical NOT expression, which can include a NOT operator applied to a comparison expression.
        
        Returns:
            Expr: The parsed logical NOT expression node.
            
        Raises:
            SyntaxError: If the logical NOT expression is invalid or unexpected.
        """
        
        if self.match(TokenKind.OP, "!"):
            self.advance()
            return UnaryOp(op="not", operand=self.parse_not())
        if self.match(TokenKind.KEYWORD, "not"):
            self.advance()
            return UnaryOp(op="not", operand=self.parse_not())
        return self.parse_comparison()

    def parse_comparison(self) -> Expr:
        """
        Parse a comparison expression, which can include multiple comparison operators.
        
        Returns:
            Expr: The parsed comparison expression node.
            
        Raises:
            SyntaxError: If the comparison expression is invalid or unexpected.
        """
        
        left = self.parse_bitwise_or()
        ops: List[str] = []
        comparators: List[Expr] = []
        while True:
            if self.match(TokenKind.OP) and self.token.value in {"==", "!=", "<", ">", "<=", ">="}:
                op = self.token.value
                self.advance()
            elif self.match(TokenKind.KEYWORD, "is"):
                self.advance()
                if self.match(TokenKind.KEYWORD, "not"):
                    op = "is not"
                    self.advance()
                else:
                    op = "is"
            elif self.match(TokenKind.KEYWORD, "not") and self._peek().kind == TokenKind.KEYWORD and self._peek().value == "in":
                op = "not in"
                self.advance()
                self.advance()
            elif self.match(TokenKind.KEYWORD, "in"):
                op = "in"
                self.advance()
            else:
                break
            ops.append(op)
            comparators.append(self.parse_bitwise_or())
        if ops:
            return Compare(left=left, ops=ops, comparators=comparators)
        return left

    def parse_if_expression(self, test: Expr) -> Expr:
        """
        Parse an if expression, which includes a test, a body, and an else clause.
        
        Returns:
            Expr: The parsed if expression node.
            
        Raises:
            SyntaxError: If the if expression is invalid or unexpected.
        """
        
        body = self.parse_expression()
        self.expect(TokenKind.KEYWORD, "else")
        orelse = self.parse_expression()
        return IfExpr(test=test, body=body, orelse=orelse)

    def parse_bitwise_or(self) -> Expr:
        """
        Parse a bitwise OR expression, which includes the '|' operator.

        Returns:
            Expr: The parsed bitwise OR expression node.

        Raises:
            SyntaxError: If the bitwise OR expression is invalid or unexpected.
        """
        
        left = self.parse_bitwise_xor()
        while self.match(TokenKind.OP, "|"):
            self.advance()
            right = self.parse_bitwise_xor()
            left = BinOp(left=left, op="|", right=right)
        return left

    def parse_bitwise_xor(self) -> Expr:
        """
        Parse a bitwise XOR expression, which includes the '^' operator.

        Returns:
            Expr: The parsed bitwise XOR expression node.

        Raises:
            SyntaxError: If the bitwise XOR expression is invalid or unexpected.
        """
        
        left = self.parse_bitwise_and()
        while self.match(TokenKind.OP, "^"):
            self.advance()
            right = self.parse_bitwise_and()
            left = BinOp(left=left, op="^", right=right)
        return left

    def parse_bitwise_and(self) -> Expr:
        """
        Parse a bitwise AND expression, which includes the '&' operator.
        
        Returns:
            Expr: The parsed bitwise AND expression node.

        Raises:
            SyntaxError: If the bitwise AND expression is invalid or unexpected.
        """
        
        left = self.parse_shift()
        while self.match(TokenKind.OP, "&"):
            self.advance()
            right = self.parse_shift()
            left = BinOp(left=left, op="&", right=right)
        return left

    def parse_shift(self) -> Expr:
        """
        Parse a shift expression, which includes left and right bitwise shifts.

        Returns:
            Expr: The parsed shift expression node.

        Raises:
            SyntaxError: If the shift expression is invalid or unexpected.
        """
        
        left = self.parse_term()
        while self.match(TokenKind.OP) and self.token.value in {"<<", ">>"}:
            op = self.token.value
            self.advance()
            right = self.parse_term()
            left = BinOp(left=left, op=op, right=right)
        return left

    def parse_term(self) -> Expr:
        """
        Parse a term expression, which includes addition and subtraction.

        Returns:
            Expr: The parsed term expression node.

        Raises:
            SyntaxError: If the term expression is invalid or unexpected.
        """
        
        left = self.parse_factor()
        while self.match(TokenKind.OP) and self.token.value in {"+", "-"}:
            op = self.token.value
            self.advance()
            right = self.parse_factor()
            left = BinOp(left=left, op=op, right=right)
        return left

    def parse_factor(self) -> Expr:
        """
        Parse a factor expression, which includes unary plus and minus operators, as well as multiplication, division, modulo, and floor division.

        Returns:
            Expr: The parsed factor expression node.

        Raises:
            SyntaxError: If the factor expression is invalid or unexpected.
        """
        
        if self.match(TokenKind.OP) and self.token.value in {"+", "-", "~"}:
            op = self.token.value
            self.advance()
            operand = self.parse_factor()
            return UnaryOp(op=op, operand=operand)
        left = self.parse_power()
        while self.match(TokenKind.OP) and self.token.value in {"*", "/", "%", "//"}:
            op = self.token.value
            self.advance()
            right = self.parse_power()
            left = BinOp(left=left, op=op, right=right)
        return left

    def parse_power(self) -> Expr:
        """
        Parse a power expression, which includes exponentiation.

        Returns:
            Expr: The parsed power expression node.

        Raises:
            SyntaxError: If the power expression is invalid or unexpected.
        """
        
        left = self.parse_atom()
        while self.match(TokenKind.OP) and self.token.value == "**":
            self.advance()
            right = self.parse_factor()
            left = BinOp(left=left, op="**", right=right)
        return left

    def parse_atom(self) -> Expr:
        """
        Parse an atomic expression, which includes literals, names, and parenthesized expressions.

        Returns:
            Expr: The parsed atomic expression node.

        Raises:
            SyntaxError: If the atomic expression is invalid or unexpected.
        """
        
        # Handle numeric literals (integers, floats, etc.)
        if self.match(TokenKind.NUMBER):
            value = self.token.value
            self.advance()
            return Constant(value=value)

        # Handle string literals, including concatenation of adjacent strings
        if self.match(TokenKind.STRING):
            value = self.token.value
            if not isinstance(value, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            self.advance()
            while self.match(TokenKind.STRING):
                next_value = self.token.value
                if not isinstance(next_value, str):
                    raise E01Error(
                        token_type=self.token.kind.name,
                        actual_type=type(self.token.value).__name__,
                        line_number=self.token.line
                    )
                value = value + next_value
                self.advance()
            node = Constant(value=value)
            return self.parse_trailer(node)

        # Handle f-string literals, which are formatted string literals
        if self.match(TokenKind.FSTRING):
            value = self.token.value
            if not isinstance(value, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            self.advance()
            node = FString(value=value)
            return self.parse_trailer(node)

        # Handle boolean and None literals
        if self.match(TokenKind.KEYWORD) and self.token.value in {"True", "False", "None"}:
            raw = self.token.value
            self.advance()
            if raw == "True":
                return Constant(value=True)
            if raw == "False":
                return Constant(value=False)
            return Constant(value=None)

        # Handle lambda expressions (anonymous functions)
        if self.match(TokenKind.KEYWORD, "func"):
            node = self.parse_lambda()
            return self.parse_trailer(node)

        # Handle yield expressions, which can yield values from a generator function
        if self.match(TokenKind.KEYWORD, "yield"):
            self.advance()
            if self.match(TokenKind.SEMICOLON) or self.match(TokenKind.RPAREN) or self.match(TokenKind.RBRACKET) or self.match(TokenKind.RBRACE):
                return Yield(value=None)
            return Yield(value=self.parse_expression())

        # Handle await expressions, which can await values from an asynchronous function
        if self.match(TokenKind.KEYWORD, "await"):
            self.advance()
            return Await(value=self.parse_expression())

        # Handle macro calls
        if self.match(TokenKind.DOLLAR):
            return self.parse_macro_call()

        # Handle names (identifiers), which can be variables, functions, or attributes
        if self.match(TokenKind.NAME):
            id = self.token.value
            if not isinstance(id, str):
                raise E01Error(
                    token_type=self.token.kind.name,
                    actual_type=type(self.token.value).__name__,
                    line_number=self.token.line
                )
            node = Name(id=id)
            self.advance()
            return self.parse_trailer(node)

        # Handle list literals and list comprehensions
        if self.match(TokenKind.LBRACKET):
            self.advance()
            elements: List[Expr] = []
            if not self.match(TokenKind.RBRACKET):
                first = self.parse_expression()
                if self.match(TokenKind.KEYWORD, "for"):
                    generators = self.parse_comprehension_generators()
                    self.expect(TokenKind.RBRACKET)
                    return ListComp(elt=first, generators=generators)
                elements.append(first)
                while self.match(TokenKind.COMMA):
                    self.advance()
                    if self.match(TokenKind.RBRACKET):
                        break
                    elements.append(self.parse_expression())
            self.expect(TokenKind.RBRACKET)
            return ListExpr(elements=elements)

        # Handle brace literals, which can represent dictionaries, sets, or comprehensions
        if self.match(TokenKind.LBRACE):
            return self.parse_brace_literal()

        # Handle parenthesized expressions, which can include tuples, generator expressions, and if expressions
        if self.match(TokenKind.LPAREN):
            self.advance()
            if self.match(TokenKind.RPAREN):
                self.advance()
                return self.parse_trailer(TupleExpr(elements=[]))
            expr = self.parse_expression()
            if self.match(TokenKind.KEYWORD, "for"):
                generators = self.parse_comprehension_generators()
                self.expect(TokenKind.RPAREN)
                return self.parse_trailer(GeneratorExp(elt=expr, generators=generators))
            if self.match(TokenKind.KEYWORD, "if"):
                self.advance()
                if_expr = self.parse_if_expression(expr)
                self.expect(TokenKind.RPAREN)
                return self.parse_trailer(if_expr)
            if self.match(TokenKind.COMMA):
                elements = [expr]
                while self.match(TokenKind.COMMA):
                    self.advance()
                    if self.match(TokenKind.RPAREN):
                        break
                    elements.append(self.parse_expression())
                self.expect(TokenKind.RPAREN)
                return self.parse_trailer(TupleExpr(elements=elements))
            self.expect(TokenKind.RPAREN)
            return self.parse_trailer(expr)
        raise SyntaxError(
            f"Unexpected token {self.token.kind} {self.token.value!r} at line {self.token.line}, col {self.token.column}"
        )

    def parse_trailer(self, node: Expr) -> Expr:
        """
        Parse a trailer, which can include function calls, subscripts, and attribute access.

        Args:
            node (Expr): The expression node to which the trailer is applied.

        Returns:
            Expr: The parsed expression node with the applied trailer.

        Raises:
            SyntaxError: If the trailer is invalid or unexpected.
        """
        while True:
            if self.match(TokenKind.LPAREN):
                self.advance()
                args: List[Expr] = []
                keywords: List[tuple[str, Expr]] = []
                starargs: List[Expr] = []
                kwargs: List[Expr] = []
                while not self.match(TokenKind.RPAREN):
                    if self.match(TokenKind.SEMICOLON):
                        self.advance()
                        continue
                    if self.match(TokenKind.OP, "**"):
                        self.advance()
                        kwargs.append(self.parse_expression())
                    elif self.match(TokenKind.OP, "*"):
                        self.advance()
                        starargs.append(self.parse_expression())
                    else:
                        expr = self.parse_expression()
                        if self.match(TokenKind.OP, "=") and isinstance(expr, Name):
                            self.advance()
                            value = self.parse_expression()
                            keywords.append((expr.id, value))
                        else:
                            args.append(expr)
                    if self.match(TokenKind.SEMICOLON):
                        self.advance()
                        continue
                    if self.match(TokenKind.COMMA):
                        self.advance()
                        continue
                    break
                self.expect(TokenKind.RPAREN)
                node = Call(func=node, args=args, keywords=keywords, starargs=starargs, kwargs=kwargs)
                continue
            if self.match(TokenKind.LBRACKET):
                self.advance()
                slice_expr = self.parse_slice()
                self.expect(TokenKind.RBRACKET)
                node = Subscript(value=node, slice=slice_expr)
                continue
            if self.match(TokenKind.OP, "<"):
                self.advance()
                elements: List[Expr] = []
                if not self.match(TokenKind.OP, ">"):
                    elements.append(self.parse_expression())
                    while self.match(TokenKind.COMMA):
                        self.advance()
                        if self.match(TokenKind.OP, ">"):
                            break
                        elements.append(self.parse_expression())
                self.expect(TokenKind.OP, ">")
                slice_expr = TupleExpr(elements=elements) if len(elements) != 1 else elements[0]
                node = Subscript(value=node, slice=slice_expr)
                continue
            if self.match(TokenKind.DOT):
                self.advance()
                attr = self.expect(TokenKind.NAME).value
                if not isinstance(attr, str):
                    raise E01Error(
                        token_type=self.token.kind.name,
                        actual_type=type(self.token.value).__name__,
                        line_number=self.token.line
                    )
                node = Attribute(value=node, attr=attr)
                continue
            break
        return node

    def _peek(self) -> Token:
        """
        Peek at the next token without advancing the position.

        Returns:
            Token: The next token in the token stream.
        """
        return self.tokens[-1]
