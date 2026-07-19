import std/[json, tables, strutils]
import visitor
import ../[astnodes, tokens, types]

type
  ASTPrinterVisitor* = ref object of Visitor
    output*: string

proc newASTPrinterVisitor*(): ASTPrinterVisitor =
  ASTPrinterVisitor(output: "")

method visitExpression*(visitor: ASTPrinterVisitor, node: Expression): JsonNode {.base.}

method visitErrorExpression*(visitor: ASTPrinterVisitor, node: ErrorExpression): JsonNode {.base.} =
  %*{"kind": "ErrorExpression", "token": node.token.mean()}

method visitNumberExpression*(visitor: ASTPrinterVisitor, node: NumberExpression): JsonNode {.base.} =
  %*{"kind": "IntExpression", "value": node.token.lexeme}

method visitBoolExpression*(visitor: ASTPrinterVisitor, node: BoolExpression): JsonNode {.base.} =
  %*{"kind": "BoolExpression", "value": node.token.lexeme}

method visitBinaryExpression*(visitor: ASTPrinterVisitor, node: BinaryExpression): JsonNode {.base.} =
  %*{
    "kind": "BinaryExpression",
    "op": node.token.lexeme,
    "left": visitor.visitExpression(node.left),
    "right": visitor.visitExpression(node.right)
  }

method visitUnaryExpression*(visitor: ASTPrinterVisitor, node: UnaryExpression): JsonNode {.base.} =
  %*{
    "kind": "UnaryExpression",
    "op": node.token.lexeme,
    "operand": visitor.visitExpression(node.operand)
  }

method visitIdentifierExpression*(visitor: ASTPrinterVisitor, node: IdentifierExpression): JsonNode {.base.} =
  %*{"kind": "IdentifierExpression", "name": node.token.lexeme}

method visitCastExpression*(visitor: ASTPrinterVisitor, node: CastExpression): JsonNode {.base.} =
  %*{
    "kind": "CastExpression",
    "to": $node.returnType,
    "value": visitor.visitExpression(node.value)
  }

method visitDerefExpression*(visitor: ASTPrinterVisitor, node: DerefExpression): JsonNode {.base.} =
  %*{
    "kind": "DerefExpression",
    "operand": visitor.visitExpression(node.operand)
  }

method visitCharExpression*(visitor: ASTPrinterVisitor, node: CharExpression): JsonNode {.base.} =
  %*{"kind": "CharExpression", "value": node.token.lexeme}

method visitArrayExpression*(visitor: ASTPrinterVisitor, node: ArrayExpression): JsonNode {.base.} =
  if node.returnType.kind == typeArray and node.returnType.arrayBaseType == getCharType():
    var s = ""
    for val in node.values:
      s.add(CharExpression(val).token.lexeme)
    %*{"kind": "ArrayExpression", "value": s, "isString": true}
  else:
    var elems = newJArray()
    for val in node.values:
      elems.add(visitor.visitExpression(val))
    %*{"kind": "ArrayExpression", "elements": elems}

method visitIndexExpression*(visitor: ASTPrinterVisitor, node: IndexExpression): JsonNode {.base.} =
  %*{
    "kind": "IndexExpression",
    "operand": visitor.visitExpression(node.operand),
    "index": visitor.visitExpression(node.index)
  }

method visitNulExpression*(visitor: ASTPrinterVisitor, node: NulExpression): JsonNode {.base.} =
  %*{"kind": "NulExpression"}

method visitTypeExpression*(visitor: ASTPrinterVisitor, node: TypeExpression): JsonNode {.base.} =
  %*{"kind": "TypeExpression", "type": $node.returnType}

# STATEMENTS

method visitStatement*(visitor: ASTPrinterVisitor, node: Statement): JsonNode {.base.}

method visitWhileStatement*(visitor: ASTPrinterVisitor, node: WhileStatement): JsonNode {.base.} =
  %*{
    "kind": "WhileStatement",
    "condition": visitor.visitExpression(node.condition),
    "body": visitor.visitStatement(node.whileBlock)
  }

method visitBreakStatement*(visitor: ASTPrinterVisitor, node: BreakStatement): JsonNode {.base.} =
  %*{"kind": "BreakStatement"}

method visitContinueStatement*(visitor: ASTPrinterVisitor, node: ContinueStatement): JsonNode {.base.} =
  %*{"kind": "ContinueStatement"}

method visitDeclarationStatement*(visitor: ASTPrinterVisitor, node: DeclarationStatement): JsonNode {.base.} =
  %*{
    "kind": "DeclarationStatement",
    "type": $node.varType,
    "name": node.name.lexeme,
    "value": visitor.visitExpression(node.value)
  }

method visitAssignmentStatement*(visitor: ASTPrinterVisitor, node: AssignmentStatement): JsonNode {.base.} =
  %*{
    "kind": "AssignmentStatement",
    "left": visitor.visitExpression(node.left),
    "value": visitor.visitExpression(node.value)
  }

method visitBlockStatement*(visitor: ASTPrinterVisitor, node: BlockStatement): JsonNode {.base.} =
  var stmts = newJArray()
  for stmt in node.statements:
    stmts.add(visitor.visitStatement(stmt))
  %*{"kind": "BlockStatement", "statements": stmts}

method visitErrorStatement*(visitor: ASTPrinterVisitor, node: ErrorStatement): JsonNode {.base.} =
  %*{"kind": "ErrorStatement", "token": node.token.mean()}

method visitBranchingStatement*(visitor: ASTPrinterVisitor, node: BranchingStatement): JsonNode {.base.} =
  result = %*{
    "kind": "BranchingStatement",
    "condition": visitor.visitExpression(node.condition),
    "ifBlock": visitor.visitStatement(node.ifBlock)
  }
  
  var elifs = newJArray()
  for el in node.elifBlocks:
    elifs.add(%*{
      "condition": visitor.visitExpression(el.cond),
      "block": visitor.visitStatement(el.elifBlock)
    })
  result["elifBlocks"] = elifs
  
  if node.elseBlock != nil:
    result["elseBlock"] = visitor.visitStatement(node.elseBlock)
  
  return result

method visitDefaultStatement*(visitor: ASTPrinterVisitor, node: DefaultStatement): JsonNode {.base.} =
  %*{
    "kind": "DefaultStatement",
    "type": $node.varType,
    "name": node.name.lexeme
  }

# SPECIALS

method visitSpecialExpression*(visitor: ASTPrinterVisitor, node: SpecialExpression): JsonNode {.base.} =
  var args = newJObject()
  for token, expr in node.namedArgs:
    if token.kind == tkNumber:
      let pos = parseInt(token.lexeme)
      args[$pos] = visitor.visitExpression(expr)
    else:
      args[token.lexeme] = visitor.visitExpression(expr)
  %*{"kind": "SpecialExpression", "special": $node.kind, "args": args}

method visitSpecialStatement*(visitor: ASTPrinterVisitor, node: SpecialStatement): JsonNode {.base.} =
  var args = newJObject()
  for token, expr in node.namedArgs:
    if token.kind == tkNumber:
      let pos = parseInt(token.lexeme)
      args[$pos] = visitor.visitExpression(expr)
    else:
      args[token.lexeme] = visitor.visitExpression(expr)
  %*{"kind": "SpecialStatement", "special": $node.kind, "args": args}

# GENERAL

method visitExpression*(visitor: ASTPrinterVisitor, node: Expression): JsonNode {.base.} =
  if node of ErrorExpression:
    return visitor.visitErrorExpression(ErrorExpression(node))
  elif node of NumberExpression:
    return visitor.visitNumberExpression(NumberExpression(node))
  elif node of BoolExpression:
    return visitor.visitBoolExpression(BoolExpression(node))
  elif node of BinaryExpression:
    return visitor.visitBinaryExpression(BinaryExpression(node))
  elif node of UnaryExpression:
    return visitor.visitUnaryExpression(UnaryExpression(node))
  elif node of IdentifierExpression:
    return visitor.visitIdentifierExpression(IdentifierExpression(node))
  elif node of CastExpression:
    return visitor.visitCastExpression(CastExpression(node))
  elif node of DerefExpression:
    return visitor.visitDerefExpression(DerefExpression(node))
  elif node of CharExpression:
    return visitor.visitCharExpression(CharExpression(node))
  elif node of ArrayExpression:
    return visitor.visitArrayExpression(ArrayExpression(node))
  elif node of IndexExpression:
    return visitor.visitIndexExpression(IndexExpression(node))
  elif node of NulExpression:
    return visitor.visitNulExpression(NulExpression(node))
  elif node of SpecialExpression:
    return visitor.visitSpecialExpression(SpecialExpression(node))
  elif node of TypeExpression:
    return visitor.visitTypeExpression(TypeExpression(node))
  else:
    echo "[ASTPrinterVisitor] WARNING: unhandled expression"
    return %*{"kind": "UNHANDLED_EXPRESSION"}

method visitStatement*(visitor: ASTPrinterVisitor, node: Statement): JsonNode {.base.} =
  if node of DeclarationStatement:
    return visitor.visitDeclarationStatement(DeclarationStatement(node))
  elif node of BlockStatement:
    return visitor.visitBlockStatement(BlockStatement(node))
  elif node of ErrorStatement:
    return visitor.visitErrorStatement(ErrorStatement(node))
  elif node of AssignmentStatement:
    return visitor.visitAssignmentStatement(AssignmentStatement(node))
  elif node of BranchingStatement:
    return visitor.visitBranchingStatement(BranchingStatement(node))
  elif node of SpecialStatement:
    return visitor.visitSpecialStatement(SpecialStatement(node))
  elif node of BreakStatement:
    return visitor.visitBreakStatement(BreakStatement(node))
  elif node of ContinueStatement:
    return visitor.visitContinueStatement(ContinueStatement(node))
  elif node of WhileStatement:
    return visitor.visitWhileStatement(WhileStatement(node))
  elif node of DefaultStatement:
    return visitor.visitDefaultStatement(DefaultStatement(node))
  else:
    echo "[ASTPrinterVisitor] WARNING: unhandled statement"
    return %*{"kind": "UNHANDLED_STATEMENT"}

proc printStatement*(visitor: ASTPrinterVisitor, node: Statement): string =
  visitor.output = ""
  return $visitor.visitStatement(node)