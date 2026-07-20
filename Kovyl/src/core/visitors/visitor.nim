import ../astnodes

type
  Visitor* = ref object of RootObj

method visitErrorExpression*(visitor: Visitor, node: ErrorExpression): auto {.base.} =
  discard

method visitNumberExpression*(visitor: Visitor, node: NumberExpression): auto {.base.} =
  discard

method visitBoolExpression*(visitor: Visitor, node: BoolExpression): auto {.base.} =
  discard

method visitBinaryExpression*(visitor: Visitor, node: BinaryExpression): auto {.base.} =
  discard

method visitUnaryExpression*(visitor: Visitor, node: UnaryExpression): auto {.base.} =
  discard

method visitIdentifierExpression*(visitor: Visitor, node: IdentifierExpression): auto {.base.} =
  discard

method visitCastExpression*(visitor: Visitor, node: CastExpression): auto {.base.} =
  discard

method visitDerefExpression*(visitor: Visitor, node: DerefExpression): auto {.base.} =
  discard

method visitCharExpression*(visitor: Visitor, node: CharExpression): auto {.base.} =
  discard

method visitArrayExpression*(visitor: Visitor, node: ArrayExpression): auto {.base.} =
  discard

method visitIndexExpression*(visitor: Visitor, node: IndexExpression): auto {.base.} =
  discard

method visitNulExpression*(visitor: Visitor, node: NulExpression): auto {.base.} =
  discard

method visitTypeExpression*(visitor: Visitor, node: TypeExpression): auto {.base.} =
  discard

method visitTupleExpression*(visitor: Visitor, node: TupleExpression): auto {.base.} =
  discard

method visitFieldExpression*(visitor: Visitor, node: FieldExpression): auto {.base.} =
  discard

method visitCallExpression*(visitor: Visitor, node: CallExpression): auto {.base.} =
  discard

# STATEMENTS

method visitWhileStatement*(visitor: Visitor, node: WhileStatement): auto {.base.} =
  discard

method visitBreakStatement*(visitor: Visitor, node: BreakStatement): auto {.base.} =
  discard

method visitContinueStatement*(visitor: Visitor, node: ContinueStatement): auto {.base.} =
  discard

method visitDeclarationStatement*(visitor: Visitor, node: DeclarationStatement): auto {.base.} =
  discard

method visitBlockStatement*(visitor: Visitor, node: BlockStatement): auto {.base.} =
  discard

method visitErrorStatement*(visitor: Visitor, node: ErrorStatement): auto {.base.} =
  discard

method visitAssignmentStatement*(visitor: Visitor, node: AssignmentStatement): auto {.base.} =
  discard

method visitBranchingStatement*(visitor: Visitor, node: BranchingStatement): auto {.base.} =
  discard

method visitDefaultStatement*(visitor: Visitor, node: DefaultStatement): auto {.base.} =
  discard

method visitFuncStatement*(visitor: Visitor, node: FuncStatement): auto {.base.} =
  discard

method visitReturnStatement*(visitor: Visitor, node: ReturnStatement): auto {.base.} =
  discard

method visitForStatement*(visitor: Visitor, node: ForStatement): auto {.base.} =
  discard

method visitCallStatement*(visitor: Visitor, node: CallStatement): auto {.base.} =
  discard

# SPECIALS

method visitSpecialExpression*(visitor: Visitor, node: SpecialExpression): auto {.base.} =
  discard

method visitSpecialStatement*(visitor: Visitor, node: SpecialStatement): auto {.base.} =
  discard