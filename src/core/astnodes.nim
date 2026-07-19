import types, tokens, errors
import std/tables

type
  # EXPRESSIONS

  Expression* = ref object of RootObj
    returnType*: Type
    token*: Token

  ErrorExpression* = ref object of Expression

  NumberExpression* = ref object of Expression

  BoolExpression* = ref object of Expression

  BinaryExpression* = ref object of Expression
    left*: Expression
    right*: Expression

  UnaryExpression* = ref object of Expression
    value*: Expression

  IdentifierExpression* = ref object of Expression

  CastExpression* = ref object of Expression
    value*: Expression

  DerefExpression* = ref object of Expression
    value*: Expression

  CharExpression* = ref object of Expression

  ArrayExpression* = ref object of Expression
    values*: seq[Expression]

  IndexExpression* = ref object of Expression
    value*: Expression
    index*: Expression

  NulExpression* = ref object of Expression

  TypeExpression* = ref object of Expression

  TupleExpression* = ref object of Expression
    elements*: OrderedTable[Token, Expression]

  FieldExpression* = ref object of Expression
    value*: Expression
    field*: Token

  CallExpression* = ref object of Expression
    value*: Expression
    arguments*: seq[Expression]

  # STATEMENTS

  Statement* = ref object of RootObj

  DeclarationStatement* = ref object of Statement
    symbolType*: Type
    name*: Token
    value*: Expression

  BlockStatement* = ref object of Statement
    startToken*: Token
    endToken*: Token
    statements*: seq[Statement]

  AssignmentStatement* = ref object of Statement
    left*: Expression
    value*: Expression

  ErrorStatement* = ref object of Statement
    token*: Token

  BranchingStatement* = ref object of Statement
    condition*: Expression
    ifBlock*: BlockStatement
    elifBlocks*: seq[tuple[cond: Expression, elifBlock: BlockStatement]]
    elseBlock*: BlockStatement

  WhileStatement* = ref object of Statement
    token*: Token
    condition*: Expression
    whileBlock*: BlockStatement

  BreakStatement* = ref object of Statement
    token*: Token

  ContinueStatement* = ref object of Statement
    token*: Token

  DefaultStatement* = ref object of Statement
    symbolType*: Type
    name*: Token

  FuncArgument* = object
    origin*: Token
    expectedType*: Type

  FuncStatement* = ref object of Statement
    returnType*: Type
    name*: Token
    arguments*: OrderedTable[string, FuncArgument]
    funcBlock*: BlockStatement
    funcType*: Type

  ReturnStatement* = ref object of Statement
    token*: Token
    case hasValue*: bool
    of true: value*: Expression
    else: discard

  ForStatement* = ref object of Statement
    token*: Token
    name*: Token
    value*: Expression
    forBlock*: BlockStatement

  CallStatement* = ref object of Statement
    callExpression*: CallExpression

  # SPECIALS

  SpecialExprKind* = enum
    skExprError
    skNew, skArr, skLen, skFmt, skTake, skTakeof, skJoin, skRead

  SpecialExpression* = ref object of Expression
    kind*: SpecialExprKind
    namedArgs*: OrderedTable[Token, Expression]

  SpecialStmtKind* = enum
    skStmtError
    skPrint, skFree, skAssert, skResize

  SpecialStatement* = ref object of Statement
    token*: Token
    kind*: SpecialStmtKind
    namedArgs*: OrderedTable[Token, Expression]

proc newSpecialExpression*(token: Token, kind: SpecialExprKind, namedArgs: OrderedTable[Token, Expression]): SpecialExpression =
  SpecialExpression(token: token, kind: kind, namedArgs: namedArgs, returnType: getUndefinedType())

proc newSpecialStatement*(token: Token, kind: SpecialStmtKind, namedArgs: OrderedTable[Token, Expression]): SpecialStatement =
  SpecialStatement(token: token, kind: kind, namedArgs: namedArgs)

proc getSpecialExprKind*(token: Token): SpecialExprKind =
  case token.lexeme
  of "new": skNew
  of "arr": skArr
  of "len": skLen
  of "fmt": skFmt
  of "take": skTake
  of "takeof": skTakeof
  of "join": skJoin
  of "read": skRead
  else:
    newError(errExprSpecial, token)
    return skExprError

proc getSpecialStmtKind*(token: Token): SpecialStmtKind =
  case token.lexeme
  of "print": skPrint
  of "free": skFree
  of "assert": skAssert
  of "resize": skResize
  else:
    newError(errStmtSpecial, token)
    return skStmtError

# EXPRESSIONS

proc newCallExpression*(token: Token, value: Expression, arguments: seq[Expression]): CallExpression =
  CallExpression(token: token, value: value, arguments: arguments, returnType: getUndefinedType())

proc newFieldExpression*(token: Token, value: Expression, field: Token): FieldExpression {.inline.} =
  FieldExpression(token: token, value: value, field: field, returnType: getUndefinedType())

proc newTupleExpression*(token: Token, elements: OrderedTable[Token, Expression]): TupleExpression {.inline.} =
  TupleExpression(token: token, returnType: getUndefinedType(), elements: elements)

proc newTypeExpression*(token: Token, returnType: Type): TypeExpression =
  TypeExpression(token: token, returnType: returnType)

proc newNulExpression*(token: Token): NulExpression {.inline.} =
  NulExpression(token: token, returnType: getNulType())

proc newIndexExpression*(token: Token, value: Expression, index: Expression): IndexExpression {.inline.} =
  IndexExpression(token: token, value: value, index: index, returnType: getUndefinedType())

proc newArrayExpression*(token: Token): ArrayExpression {.inline.} =
  ArrayExpression(token: token, returnType: getUndefinedType())

proc addExpr*(self: var ArrayExpression, expr: Expression) {.inline.} =
  self.values.add(expr)

proc newCharExpression*(token: Token): CharExpression {.inline.} =
  CharExpression(token: token, returnType: getCharType())

proc newDerefExpression*(token: Token, value: Expression): DerefExpression {.inline.} =
  DerefExpression(token: token, value: value, returnType: getUndefinedType())

proc newCastExpression*(castToken: Token, castType: Type, value: Expression): CastExpression {.inline.} =
  CastExpression(token: castToken, returnType: castType, value: value)

proc newErrorExpression*(token: Token): ErrorExpression {.inline.} =
  ErrorExpression(token: token, returnType: getUndefinedType())

proc newNumberExpression*(value: Token): NumberExpression {.inline.} =
  NumberExpression(token: value, returnType: getInt64Type())

proc newBoolExpression*(value: Token): BoolExpression {.inline.} =
  BoolExpression(token: value, returnType: getBoolType())

proc newBinaryExpression*(left: Expression, op: Token, right: Expression): BinaryExpression {.inline.} =
  BinaryExpression(left: left, token: op, right: right, returnType: getUndefinedType())

proc newUnaryExpression*(value: Expression, op: Token): UnaryExpression {.inline.} =
  UnaryExpression(value: value, token: op, returnType: getUndefinedType())

proc newIdentifierExpression*(name: Token): IdentifierExpression {.inline.} =
  IdentifierExpression(token: name, returnType: getUndefinedType())

# STATEMENTS

proc newCallStatement*(callExpression: CallExpression): CallStatement {.inline.} =
  CallStatement(callExpression: callExpression)

proc newForStatement*(token: Token, name: Token, value: Expression, forBlock: BlockStatement): ForStatement {.inline.} =
  ForStatement(token: token, name: name, value: value, forBlock: forBlock)

proc newReturnStatement*(token: Token, hasValue: bool, value: Expression = nil): ReturnStatement {.inline.} =
  if hasValue: ReturnStatement(token: token, hasValue: true, value: value)
  else: ReturnStatement(token: token, hasValue: false)

proc newFuncStatement*(returnType: Type, name: Token, arguments: OrderedTable[string, FuncArgument], 
    funcBlock: BlockStatement): FuncStatement {.inline.} =
  FuncStatement(returnType: returnType, name: name, arguments: arguments, funcBlock: funcBlock,
    funcType: getUndefinedType())

proc newFuncArgument*(origin: Token, expectedType: Type): FuncArgument =
  FuncArgument(origin: origin, expectedType: expectedType)

proc newWhileStatement*(token: Token, condition: Expression, whileBlock: BlockStatement): WhileStatement {.inline.} =
  WhileStatement(token: token, condition: condition, whileBlock: whileBlock)

proc newBreakStatement*(token: Token): BreakStatement {.inline.} =
  BreakStatement(token: token)

proc newContinueStatement*(token: Token): ContinueStatement {.inline.} =
  ContinueStatement(token: token)

proc newBranchingStatement*(condition: Expression, ifBlock: BlockStatement): BranchingStatement {.inline.} =
  BranchingStatement(condition: condition, ifBlock: ifBlock, elifBlocks: @[], elseBlock: nil)

proc addElif*(self: var BranchingStatement, condition: Expression, elifBlock: BlockStatement) {.inline.} =
  self.elifBlocks.add((condition, elifBlock))

proc setElse*(self: var BranchingStatement, elseBlock: BlockStatement) {.inline.} =
  self.elseBlock = elseBlock

proc newAssignmentStatement*(left: Expression, value: Expression): AssignmentStatement {.inline.} =
  AssignmentStatement(left: left, value: value)

proc newErrorStatement*(token: Token): ErrorStatement {.inline.} =
  ErrorStatement(token: token)

proc newBlockStatement*(startToken: Token, endToken: Token): BlockStatement {.inline.} =
  BlockStatement(startToken: startToken, endToken: endToken, statements: @[])

proc newBlockStatement*(startToken: Token): BlockStatement {.inline.} =
  BlockStatement(startToken: startToken, endToken: tkInvalid.newToken(startToken.lexeme, 
    startToken.file, startToken.line, startToken.column, startToken.offset), statements: @[])

proc addStatement*(blockStmt: BlockStatement, stmt: Statement) {.inline.} =
  blockStmt.statements.add(stmt)

proc newDeclarationStatement*(
    symbolType: Type, name: Token, value: Expression
  ): DeclarationStatement {.inline.} =
  DeclarationStatement(name: name, value: value, symbolType: symbolType)

proc newDefaultStatement*(symbolType: Type, name: Token): DefaultStatement {.inline.} =
  DefaultStatement(name: name, symbolType: symbolType)