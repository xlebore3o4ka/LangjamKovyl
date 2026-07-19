import ../core/[astnodes, types, errors, tokens]
import std/[strutils, logging]

proc isNumber*(t: Type): bool {.inline.} =
  t.kind in {typeInt64, typeInt32, typeInt16, typeInt8, typeUint64, typeUint32, typeUint16, typeUint8}

proc isValidInt*[T: SomeSignedInt](s: string): bool =
  try:
    let v = parseInt(s)
    return v >= low(T) and v <= high(T)
  except ValueError:
    return false

proc isValidUint*[T: SomeUnsignedInt](s: string): bool =
  try:
    let v = parseUInt(s)
    return v <= high(T)
  except ValueError:
    return false

proc inferNumberType*(node: NumberExpression, expected: Type): Type =
  if not expected.isNumber:
    return getInt64Type()
  let number = node.token.lexeme
  if expected.eq(typeInt8) and isValidInt[int8](number): return expected
  if expected.eq(typeInt16) and isValidInt[int16](number): return expected
  if expected.eq(typeInt32) and isValidInt[int32](number): return expected
  if expected.eq(typeInt64) and isValidInt[int64](number): return expected
  if expected.eq(typeUint8) and isValidUint[uint8](number): return expected
  if expected.eq(typeUint16) and isValidUint[uint16](number): return expected
  if expected.eq(typeUint32) and isValidUint[uint32](number): return expected
  if expected.eq(typeUint64) and isValidUint[uint64](number): return expected
  newError(errSize, node.token, @{"@0": number, "@1": $expected})
  return getInt64Type()

proc setType*(expr: Expression, returnType: Type) {.inline.} =
  expr.returnType = returnType
  info("Return type is set as: ", $returnType)

proc checkEqNeq*(node: BinaryExpression, expected: TypeKind): bool {.inline.} =
  if node.token.kind notin {tkEq, tkNeq}: return false
  node.left.returnType.kind.eq(expected) and node.right.returnType.kind.eq(expected)

proc checkEqNeqStrings*(node: BinaryExpression): bool {.inline.} =
  if node.token.kind notin {tkEq, tkNeq}: return false

  let dyn = getArrayType(getCharType())
  let sta = getStaticArrayType(getCharType(), 0)
  
  if node.left.returnType.eq(dyn) and node.right.returnType.eq(dyn): return true
  if node.left.returnType.eq(dyn) and node.right.returnType.eq(sta): return true
  if node.left.returnType.eq(sta) and node.right.returnType.eq(dyn): return true
  if node.left.returnType.eq(sta) and node.right.returnType.eq(sta): return true

  return false

proc checkAndOr*(node: BinaryExpression): bool {.inline.} =
  if node.token.kind notin {tkAnd, tkOr}: return false
  node.left.returnType.kind.eq(typeBool) and node.right.returnType.kind.eq(typeBool)

proc trySetNumber*(node: BinaryExpression): bool {.inline.} =
  if node.left.returnType.isNumber and node.right.returnType.eq node.left.returnType: 
    if node.token.kind in {tkPlus, tkMinus, tkStar, tkSlash, tkPercent}: 
      node.setType(node.left.returnType); return true
    elif node.token.kind in {tkGT, tkLT, tkGTE, tkLTE, tkEQ, tkNEQ}: 
      node.setType(getBoolType()); return true
  return false

proc newBinaryTypeMismatchError*(node: BinaryExpression) {.inline.} =
  newError(errBinaryTypeMismatch, node.token, @{"@0": node.token.lexeme, "@1": $node.left.returnType, 
      "@2": $node.right.returnType})

proc checkPlusMinus*(node: UnaryExpression): bool {.inline.} =
  if node.token.kind notin {tkPlus, tkMinus}: return false
  node.value.returnType.isNumber

proc checkNot*(node: UnaryExpression): bool {.inline.} =
  if node.token.kind != tkNot: return false
  node.value.returnType.eq getBoolType()

proc newUnaryTypeMismatchError*(node: UnaryExpression) {.inline.} =
  newError(errUnaryTypeMismatch, node.token, @{"@0": node.token.lexeme, "@1": $node.value.returnType})