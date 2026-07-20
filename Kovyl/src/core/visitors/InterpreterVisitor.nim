import visitor
import std/[strutils, logging, tables]
import ../[astnodes, types, tokens]

type
  ErrorKind* = enum
    errSIGSEGV
    errArithmeticOverflow, errSign
    errIndex
    errAssert
    errArrayLengthMismatch

  RuntimeError* = object of CatchableError
    kind: ErrorKind

  VecValue* = ref object
    values: seq[Value]
    length: Natural

  FuncValue* = ref object
    arguments: OrderedTable[string, FuncArgument]
    body: BlockStatement

  Value* = object
    valueType*: Type
    case kind: TypeKind
    of typeUndefined: discard

    of typeInt64:        int64Value:       int64
    of typeInt32:        int32Value:       int32
    of typeInt16:        int16Value:       int16
    of typeInt8:         int8Value:        int8
    of typeUint64:       uint64Value:      uint64
    of typeUint32:       uint32Value:      uint32
    of typeUint16:       uint16Value:      uint16
    of typeUint8:        uint8Value:       uint8

    of typeBool:         boolValue:        bool
    of typeChar:         charValue:        char

    of typeArray:
                         arrayData:        ref seq[Value]
                         arrayLen:         Natural
        
    of typeVec:          vecValue:         VecValue

    of typePtr:          ptrValue:         ref Value
    of typeNul:          discard

    of typeTuple:        tupleValue:       OrderedTable[string, Value]
    of typeFunc:         funcValue:        FuncValue

  InterpreterVisitor* = ref object of Visitor
    environment: seq[Table[string, Value]]

  BreakException* = object of CatchableError
  ContinueException* = object of CatchableError
  ReturnException* = object of CatchableError
    value*: Value

proc newSlot*(self: InterpreterVisitor, name: string, value: Value) =
  self.environment[^1][name] = value

proc getSlot*(self: InterpreterVisitor, name: string): Value =
  for i in countdown(self.environment.high, 0):
    if name in self.environment[i]:
      return self.environment[i][name]
  warn("Undefined variable: " & name)

proc setSlot*(self: InterpreterVisitor, name: string, value: Value) =
  for i in countdown(self.environment.high, 0):
    if name in self.environment[i]:
      self.environment[i][name] = value
      return
  warn("Undefined variable: " & name)

proc pushScope*(self: InterpreterVisitor) =
  self.environment.add(initTable[string, Value]())

proc popScope*(self: InterpreterVisitor) =
  discard self.environment.pop()

proc isNumber*(t: Type): bool {.inline.} =
  t.kind in {typeInt64, typeInt32, typeInt16, typeInt8, typeUint64, typeUint32, typeUint16, typeUint8}

proc isInt*(t: Type): bool {.inline.} =
  t.kind in {typeInt64, typeInt32, typeInt16, typeInt8}

proc isUint*(t: Type): bool {.inline.} =
  t.kind in {typeUint64, typeUint32, typeUint16, typeUint8}

proc newDefaultValue*(valueType: Type): Value =
  Value(kind: valueType.kind, valueType: valueType)

proc newInt64Value*(v: int64): Value = 
  Value(kind: typeInt64, valueType: getInt64Type(), int64Value: v)

proc newInt32Value*(v: int32): Value = 
  Value(kind: typeInt32, valueType: getInt32Type(), int32Value: v)

proc newInt16Value*(v: int16): Value = 
  Value(kind: typeInt16, valueType: getInt16Type(), int16Value: v)

proc newInt8Value*(v: int8): Value = 
  Value(kind: typeInt8, valueType: getInt8Type(), int8Value: v)

proc newUint64Value*(v: uint64): Value = 
  Value(kind: typeUint64, valueType: getUint64Type(), uint64Value: v)

proc newUint32Value*(v: uint32): Value = 
  Value(kind: typeUint32, valueType: getUint32Type(), uint32Value: v)

proc newUint16Value*(v: uint16): Value = 
  Value(kind: typeUint16, valueType: getUint16Type(), uint16Value: v)

proc newUint8Value*(v: uint8): Value = 
  Value(kind: typeUint8, valueType: getUint8Type(), uint8Value: v)

proc newBoolValue*(v: bool): Value = 
  Value(kind: typeBool, valueType: getBoolType(), boolValue: v)

proc newCharValue*(v: char): Value = 
  Value(kind: typeChar, valueType: getCharType(), charValue: v)

proc newVecValue*(values: seq[Value], baseType: Type): Value =
  Value(
    kind: typeVec,
    valueType: getVecType(baseType),
    vecValue: VecValue(values: values, length: values.len)
  )

proc copyValueDeep(v: Value): Value =
  result = v
  if v.kind == typeArray:
    var newData = new seq[Value]
    for val in v.arrayData[]:
      newData[].add(copyValueDeep(val))
    result.arrayData = newData

proc newStaticArrayValue*(values: seq[Value], valType: Type, length: Natural): Value =
  var data = new seq[Value]
  for val in values:
    data[].add(copyValueDeep(val))
  Value(
    kind: typeArray,
    valueType: valType,
    arrayData: data,
    arrayLen: length
  )

proc newPtrValue*(v: ref Value, baseType: Type): Value = 
  Value(kind: typePtr, valueType: getPtrType(baseType), ptrValue: v)

proc newNulValue*(dataType: Type): Value = 
  Value(kind: typeNul, valueType: dataType)

proc newTupleValue*(dataType: Type, elements: OrderedTable[string, Value]): Value =
  Value(kind: typeTuple, valueType: dataType, tupleValue: elements)

proc newFuncValue*(valueType: Type, arguments: OrderedTable[string, FuncArgument], funcBlock: BlockStatement): Value =
  Value(kind: typeFunc, valueType: valueType, funcValue: FuncValue(arguments: arguments, body: funcBlock))

proc arrayLength*(v: Value): Natural =
  case v.kind:
  of typeArray:
    return v.arrayLen
  of typeVec:
    return v.vecValue.length
  else:
    raise newException(ValueError, "not an array")

proc arrayValues*(v: Value): seq[Value] =
  case v.kind:
  of typeArray:
    return v.arrayData[]
  of typeVec:
    return v.vecValue.values
  else:
    raise newException(ValueError, "not an array")

proc stringValue*(v: Value): string =
  result = ""
  case v.kind:
  of typeArray:
    for ch in v.arrayData[]:
      if ch.charValue == '\0': break
      result.add(ch.charValue)
  of typeVec:
    for ch in v.vecValue.values:
      if ch.charValue == '\0': break
      result.add(ch.charValue)
  else:
    raise newException(ValueError, "not an array")

proc newInterpreterVisitor*(): InterpreterVisitor =
  result = InterpreterVisitor()
  result.pushScope()

method visitExpression*(visitor: InterpreterVisitor, node: Expression): Value {.base.}
method visitStatement*(visitor: InterpreterVisitor, node: Statement) {.base.}

proc newError(kind: ErrorKind, msg: string = ""): ref RuntimeError =
  return (ref RuntimeError)(msg: msg, kind: kind)

var logger = newConsoleLogger(fmtStr = "KOVYL [InterpreterVisitor] $levelname: ")

proc interpreterVisitorLogging*(enabled: bool) =
  if enabled:
    logger.levelThreshold = lvlAll
    addHandler(logger)
  else:
    logger.levelThreshold = lvlNone

proc numberValue*(v: Value): int =
  case v.kind:
  of typeInt8: return int(v.int8Value)
  of typeInt16: return int(v.int16Value)
  of typeInt32: return int(v.int32Value)
  of typeInt64: return int(v.int64Value)
  of typeUint8: return int(v.uint8Value)
  of typeUint16: return int(v.uint16Value)
  of typeUint32: return int(v.uint32Value)
  of typeUint64:
    if v.uint64Value > int64.high.uint64:
      raise newError(errArithmeticOverflow, $v.uint64Value)
    return int64(v.uint64Value)
  else:
    warn("numberValue invalid type")

proc `==`*(a, b: Value): bool =
  if a.kind != b.kind:
    if a.kind.eq(typePtr) and b.kind.eq(typeNul):
      return a.ptrValue == nil
    elif a.kind.eq(typeNul) and b.kind.eq(typePtr):
      return b.ptrValue == nil
    elif a.kind.eq(typeVec) and b.kind.eq(typeNul):
      return a.vecValue == nil
    elif a.kind.eq(typeNul) and b.kind.eq(typeVec):
      return b.vecValue == nil
    elif a.valueType.eq(getVecType(getCharType())) and b.valueType.eq(getArrayType(getCharType(), 0)) or
        a.valueType.eq(getArrayType(getCharType(), 0)) and b.valueType.eq(getVecType(getCharType())):
      return a.stringValue == b.stringValue
    return false

  if a.valueType != b.valueType:
    return false

  case a.kind:
  of typeUndefined: return true
  of typeInt64:  return a.int64Value == b.int64Value
  of typeInt32:  return a.int32Value == b.int32Value
  of typeInt16:  return a.int16Value == b.int16Value
  of typeInt8:   return a.int8Value == b.int8Value
  of typeUint64: return a.uint64Value == b.uint64Value
  of typeUint32: return a.uint32Value == b.uint32Value
  of typeUint16: return a.uint16Value == b.uint16Value
  of typeUint8:  return a.uint8Value == b.uint8Value
  of typeBool:   return a.boolValue == b.boolValue
  of typeChar:   return a.charValue == b.charValue
  of typeArray:
    if a.valueType.eq(getArrayType(getCharType(), 0)) and 
      b.valueType.eq(getArrayType(getCharType(), 0)):
        return a.stringValue == b.stringValue
    if a.arrayData[].len != b.arrayData[].len:
      return false
    for i in 0..<a.arrayData[].len:
      if a.arrayData[][i] != b.arrayData[][i]:
        return false
    return true
  of typeVec:
    if a.valueType.eq(getVecType(getCharType())) and 
      b.valueType.eq(getVecType(getCharType())):
        return a.stringValue == b.stringValue
    return a.vecValue == b.vecValue
  of typePtr:
    return a.ptrValue == b.ptrValue
  of typeNul: return true
  of typeTuple:
    if a.tupleValue.len != b.tupleValue.len:
      return false
    for key, valA in a.tupleValue.pairs:
      if key notin b.tupleValue:
        return false
      if valA != b.tupleValue[key]:
        return false
    return true
  of typeFunc: 
    return a.valueType.eq b.valueType

proc `==`*(a, b: VecValue): bool =
  if a.values.len != b.values.len:
    return false
  for i in 0..<a.values.len:
    if a.values[i] != b.values[i]:
      return false
  return true

proc validIndex(index: int, arrayLength: int): int =
  result = ((index mod arrayLength) + arrayLength) mod arrayLength
  if index < -arrayLength or index > arrayLength - 1:
    raise newError(errIndex, "index " & $index & " outside the range " & 
      $(-arrayLength) & ".." & $(arrayLength - 1))

proc `$`*(value: Value): string =
  case value.kind:
  of typeInt64:  return $value.int64Value
  of typeInt32:  return $value.int32Value
  of typeInt16:  return $value.int16Value
  of typeInt8:   return $value.int8Value
  of typeUint64: return $value.uint64Value
  of typeUint32: return $value.uint32Value
  of typeUint16: return $value.uint16Value
  of typeUint8:  return $value.uint8Value
  of typeBool:   return $value.boolValue
  of typeChar:   return $value.charValue
  of typeArray:
    if value.arrayData[].len > 0 and 
       value.valueType.eq getArrayType(getCharType(), 0):
      return value.stringValue
    else:
      raise newException(ValueError, "Cannot convert static array (non-char) to string")
  of typeVec:
    if value.arrayLength > 0 and 
       value.valueType.eq getVecType(getCharType()):
      return value.stringValue
    else:
      raise newException(ValueError, "Cannot convert array (non-char) to string")
  of typePtr:
    raise newException(ValueError, "Cannot convert pointer to string")
  of typeNul:
    raise newException(ValueError, "Cannot convert nul to string")
  of typeUndefined:
    raise newException(ValueError, "Cannot convert undefined to string")
  of typeTuple:
    raise newException(ValueError, "Cannot convert tuple to string")
  of typeFunc:
    raise newException(ValueError, "Cannot convert func to string")

proc escapeString(s: string): string =
  for c in s:
    case c
    of '\0'..'\8', '\11'..'\12', '\14'..'\31':
      result.add("\\x" & c.byte.toHex(2))
    of '\n': result.add("\\n")
    of '\r': result.add("\\r")
    of '\t': result.add("\\t")
    of '\\': result.add("\\\\")
    of '\"': result.add("\\\"")
    else: result.add(c)

# EXPRESSIONS

method visitNumberExpression*(visitor: InterpreterVisitor, node: NumberExpression): Value {.base.} =
  case node.returnType.kind
  of typeInt64: return newInt64Value(int64(parseInt(node.token.lexeme)))
  of typeInt32: return newInt32Value(int32(parseInt(node.token.lexeme)))
  of typeInt16: return newInt16Value(int16(parseInt(node.token.lexeme)))
  of typeInt8: return newInt8Value(int8(parseInt(node.token.lexeme)))
  of typeUint64: return newUint64Value(uint64(parseUint(node.token.lexeme)))
  of typeUint32: return newUint32Value(uint32(parseUint(node.token.lexeme)))
  of typeUint16: return newUint16Value(uint16(parseUint(node.token.lexeme)))
  of typeUint8: return newUint8Value(uint8(parseUint(node.token.lexeme)))
  else: 
    warn("NumberExpression invalid type")

method visitBoolExpression*(visitor: InterpreterVisitor, node: BoolExpression): Value {.base.} =
  return newBoolValue(node.token.kind == tkTrue)

method visitBinaryExpression*(visitor: InterpreterVisitor, node: BinaryExpression): Value {.base.} =
  let left = visitor.visitExpression(node.left)
  let right = visitor.visitExpression(node.right)

  case node.token.kind
  of tkPlus:
    if left.valueType.eq typeInt64: return newInt64Value(left.int64Value + right.int64Value)
    elif left.valueType.eq typeInt32: return newInt32Value(left.int32Value + right.int32Value)
    elif left.valueType.eq typeInt16: return newInt16Value(left.int16Value + right.int16Value)
    elif left.valueType.eq typeInt8: return newInt8Value(left.int8Value + right.int8Value)
    elif left.valueType.eq typeUint64: return newUint64Value(left.uint64Value + right.uint64Value)
    elif left.valueType.eq typeUint32: return newUint32Value(left.uint32Value + right.uint32Value)
    elif left.valueType.eq typeUint16: return newUint16Value(left.uint16Value + right.uint16Value)
    elif left.valueType.eq typeUint8: return newUint8Value(left.uint8Value + right.uint8Value)
    else: warn("BinaryExpression invalid value type")

  of tkMinus:
    if left.valueType.eq typeInt64: return newInt64Value(left.int64Value - right.int64Value)
    elif left.valueType.eq typeInt32: return newInt32Value(left.int32Value - right.int32Value)
    elif left.valueType.eq typeInt16: return newInt16Value(left.int16Value - right.int16Value)
    elif left.valueType.eq typeInt8: return newInt8Value(left.int8Value - right.int8Value)
    elif left.valueType.eq typeUint64: return newUint64Value(left.uint64Value - right.uint64Value)
    elif left.valueType.eq typeUint32: return newUint32Value(left.uint32Value - right.uint32Value)
    elif left.valueType.eq typeUint16: return newUint16Value(left.uint16Value - right.uint16Value)
    elif left.valueType.eq typeUint8: return newUint8Value(left.uint8Value - right.uint8Value)
    else: warn("BinaryExpression invalid value type")

  of tkStar:
    if left.valueType.eq typeInt64: return newInt64Value(left.int64Value * right.int64Value)
    elif left.valueType.eq typeInt32: return newInt32Value(left.int32Value * right.int32Value)
    elif left.valueType.eq typeInt16: return newInt16Value(left.int16Value * right.int16Value)
    elif left.valueType.eq typeInt8: return newInt8Value(left.int8Value * right.int8Value)
    elif left.valueType.eq typeUint64: return newUint64Value(left.uint64Value * right.uint64Value)
    elif left.valueType.eq typeUint32: return newUint32Value(left.uint32Value * right.uint32Value)
    elif left.valueType.eq typeUint16: return newUint16Value(left.uint16Value * right.uint16Value)
    elif left.valueType.eq typeUint8: return newUint8Value(left.uint8Value * right.uint8Value)
    else: warn("BinaryExpression invalid value type")

  of tkSlash:
    if left.valueType.eq typeInt64: return newInt64Value(left.int64Value div right.int64Value)
    elif left.valueType.eq typeInt32: return newInt32Value(left.int32Value div right.int32Value)
    elif left.valueType.eq typeInt16: return newInt16Value(left.int16Value div right.int16Value)
    elif left.valueType.eq typeInt8: return newInt8Value(left.int8Value div right.int8Value)
    elif left.valueType.eq typeUint64: return newUint64Value(left.uint64Value div right.uint64Value)
    elif left.valueType.eq typeUint32: return newUint32Value(left.uint32Value div right.uint32Value)
    elif left.valueType.eq typeUint16: return newUint16Value(left.uint16Value div right.uint16Value)
    elif left.valueType.eq typeUint8: return newUint8Value(left.uint8Value div right.uint8Value)
    else: warn("BinaryExpression invalid value type")

  of tkPercent:
    if left.valueType.eq typeInt64: return newInt64Value(((left.int64Value mod right.int64Value) + right.int64Value) mod right.int64Value)
    elif left.valueType.eq typeInt32: return newInt32Value(((left.int32Value mod right.int32Value) + right.int32Value) mod right.int32Value)
    elif left.valueType.eq typeInt16: return newInt16Value(((left.int16Value mod right.int16Value) + right.int16Value) mod right.int16Value)
    elif left.valueType.eq typeInt8: return newInt8Value(((left.int8Value mod right.int8Value) + right.int8Value) mod right.int8Value)
    elif left.valueType.eq typeUint64: return newUint64Value(((left.uint64Value mod right.uint64Value) + right.uint64Value) mod right.uint64Value)
    elif left.valueType.eq typeUint32: return newUint32Value(((left.uint32Value mod right.uint32Value) + right.uint32Value) mod right.uint32Value)
    elif left.valueType.eq typeUint16: return newUint16Value(((left.uint16Value mod right.uint16Value) + right.uint16Value) mod right.uint16Value)
    elif left.valueType.eq typeUint8: return newUint8Value(((left.uint8Value mod right.uint8Value) + right.uint8Value) mod right.uint8Value)
    else: warn("BinaryExpression invalid value type")

  of tkGT:
    if left.valueType.eq typeInt64: return newBoolValue(left.int64Value > right.int64Value)
    elif left.valueType.eq typeInt32: return newBoolValue(left.int32Value > right.int32Value)
    elif left.valueType.eq typeInt16: return newBoolValue(left.int16Value > right.int16Value)
    elif left.valueType.eq typeInt8: return newBoolValue(left.int8Value > right.int8Value)
    elif left.valueType.eq typeUint64: return newBoolValue(left.uint64Value > right.uint64Value)
    elif left.valueType.eq typeUint32: return newBoolValue(left.uint32Value > right.uint32Value)
    elif left.valueType.eq typeUint16: return newBoolValue(left.uint16Value > right.uint16Value)
    elif left.valueType.eq typeUint8: return newBoolValue(left.uint8Value > right.uint8Value)
    else: warn("BinaryExpression invalid value type")

  of tkLT:
    if left.valueType.eq typeInt64: return newBoolValue(left.int64Value < right.int64Value)
    elif left.valueType.eq typeInt32: return newBoolValue(left.int32Value < right.int32Value)
    elif left.valueType.eq typeInt16: return newBoolValue(left.int16Value < right.int16Value)
    elif left.valueType.eq typeInt8: return newBoolValue(left.int8Value < right.int8Value)
    elif left.valueType.eq typeUint64: return newBoolValue(left.uint64Value < right.uint64Value)
    elif left.valueType.eq typeUint32: return newBoolValue(left.uint32Value < right.uint32Value)
    elif left.valueType.eq typeUint16: return newBoolValue(left.uint16Value < right.uint16Value)
    elif left.valueType.eq typeUint8: return newBoolValue(left.uint8Value < right.uint8Value)
    else: warn("BinaryExpression invalid value type")

  of tkGTE:
    if left.valueType.eq typeInt64: return newBoolValue(left.int64Value >= right.int64Value)
    elif left.valueType.eq typeInt32: return newBoolValue(left.int32Value >= right.int32Value)
    elif left.valueType.eq typeInt16: return newBoolValue(left.int16Value >= right.int16Value)
    elif left.valueType.eq typeInt8: return newBoolValue(left.int8Value >= right.int8Value)
    elif left.valueType.eq typeUint64: return newBoolValue(left.uint64Value >= right.uint64Value)
    elif left.valueType.eq typeUint32: return newBoolValue(left.uint32Value >= right.uint32Value)
    elif left.valueType.eq typeUint16: return newBoolValue(left.uint16Value >= right.uint16Value)
    elif left.valueType.eq typeUint8: return newBoolValue(left.uint8Value >= right.uint8Value)
    else: warn("BinaryExpression invalid value type")

  of tkLTE:
    if left.valueType.eq typeInt64: return newBoolValue(left.int64Value <= right.int64Value)
    elif left.valueType.eq typeInt32: return newBoolValue(left.int32Value <= right.int32Value)
    elif left.valueType.eq typeInt16: return newBoolValue(left.int16Value <= right.int16Value)
    elif left.valueType.eq typeInt8: return newBoolValue(left.int8Value <= right.int8Value)
    elif left.valueType.eq typeUint64: return newBoolValue(left.uint64Value <= right.uint64Value)
    elif left.valueType.eq typeUint32: return newBoolValue(left.uint32Value <= right.uint32Value)
    elif left.valueType.eq typeUint16: return newBoolValue(left.uint16Value <= right.uint16Value)
    elif left.valueType.eq typeUint8: return newBoolValue(left.uint8Value <= right.uint8Value)
    else: warn("BinaryExpression invalid value type")

  of tkEQ:
    return newBoolValue(left == right)

  of tkNeq:
    return newBoolValue(left != right)

  of tkAnd:
    return newBoolValue(left.boolValue and right.boolValue)

  of tkOr:
    return newBoolValue(left.boolValue or right.boolValue)

  else:
    warn("BinaryExpression invalid operator")

method visitUnaryExpression*(visitor: InterpreterVisitor, node: UnaryExpression): Value {.base.} =
  let value = visitor.visitExpression(node.value)

  case node.token.kind
  of tkMinus:
    if value.valueType.eq typeInt64: return newInt64Value(-value.int64Value)
    elif value.valueType.eq typeInt32: return newInt32Value(-value.int32Value)
    elif value.valueType.eq typeInt16: return newInt16Value(-value.int16Value)
    elif value.valueType.eq typeInt8: return newInt8Value(-value.int8Value)
    else: warn("UnaryExpression invalid value type")

  of tkPlus:
    if value.valueType.isNumber: return value
    else: warn("UnaryExpression invalid value type")

  of tkNot:
    if value.valueType.eq getBoolType(): return newBoolValue(not value.boolValue)
    else: warn("UnaryExpression invalid value type")

  else:
    warn("UnaryExpression invalid operator")

method visitIdentifierExpression*(visitor: InterpreterVisitor, node: IdentifierExpression): Value {.base.} =
  return visitor.getSlot(node.token.lexeme)

method visitCastExpression*(visitor: InterpreterVisitor, node: CastExpression): Value {.base.} =
  warn("CastExpression TODO")

method visitDerefExpression*(visitor: InterpreterVisitor, node: DerefExpression): Value {.base.} =
  let ptrValue = visitor.visitExpression(node.value)
  if ptrValue.kind == typeNul:
    raise newError(errSIGSEGV)
  return ptrValue.ptrValue[]

method visitCharExpression*(visitor: InterpreterVisitor, node: CharExpression): Value {.base.} =
  return newCharValue(node.token.lexeme[0])

method visitArrayExpression*(visitor: InterpreterVisitor, node: ArrayExpression): Value {.base.} =
  var values: seq[Value]
  for value in node.values:
    values.add(visitor.visitExpression(value))

  if node.values.len != node.returnType.length:
    for _ in node.values.len..node.returnType.length:
      values.add(newDefaultValue(node.returnType.arrBase))

  return newStaticArrayValue(values, node.returnType, node.returnType.length)

method visitIndexExpression*(visitor: InterpreterVisitor, node: IndexExpression): Value {.base.} =
  var index = visitor.visitExpression(node.index).numberValue
  let arr = visitor.visitExpression(node.value)
  
  let indexValue = validIndex(index, arr.arrayLength)

  case arr.kind:
  of typeArray:
    return arr.arrayData[][indexValue]
  of typeVec:
    return arr.vecValue.values[indexValue]
  else:
    raise newException(ValueError, "not an array")

method visitNulExpression*(visitor: InterpreterVisitor, node: NulExpression): Value {.base.} =
  return newNulValue(node.returnType)

method visitTupleExpression*(visitor: InterpreterVisitor, node: TupleExpression): Value {.base.} =
  var elements = initOrderedTable[string, Value]()

  for token, expr in node.elements.pairs:
    elements[token.lexeme] = visitor.visitExpression(expr)

  return newTupleValue(node.returnType, elements)

method visitFieldExpression*(visitor: InterpreterVisitor, node: FieldExpression): Value {.base.} =
  return visitor.visitExpression(node.value).tupleValue[node.field.lexeme]

method visitCallExpression*(visitor: InterpreterVisitor, node: CallExpression): Value {.base.} =
  # TODO: stacktrace
  visitor.pushScope()

  try:
    let funcValue = visitor.visitExpression(node.value).funcValue

    for index, expr in node.arguments:
      var value = visitor.visitExpression(expr)

      if value.kind == typeArray:
        value = newStaticArrayValue(value.arrayData[], value.valueType, value.arrayLen)

      visitor.newSlot(funcValue.arguments[$index].origin.lexeme, value)

    try:
      visitor.visitStatement(funcValue.body)
    except ReturnException as e:
      result = e.value

  finally:
    visitor.popScope()
    # TODO: stacktrace

# STATEMENTS

method visitBlockStatement*(visitor: InterpreterVisitor, node: BlockStatement): auto =
  for stmt in node.statements:
    visitor.visitStatement(stmt)

method visitDeclarationStatement*(visitor: InterpreterVisitor, node: DeclarationStatement): auto =
  var value = visitor.visitExpression(node.value)
  if value.kind == typeArray:
    value = newStaticArrayValue(value.arrayData[], value.valueType, value.arrayLen)
  visitor.newSlot(node.name.lexeme, value)

method visitAssignmentStatement*(visitor: InterpreterVisitor, node: AssignmentStatement): auto =
  let left = node.left
  let value = visitor.visitExpression(node.value)

  if left of IdentifierExpression:
    if value.kind == typeArray:
      var newValue = newStaticArrayValue(value.arrayData[], value.valueType, value.arrayLen)
      visitor.setSlot(left.token.lexeme, newValue)
    else:
      visitor.setSlot(left.token.lexeme, value)

  elif left of IndexExpression:
    let indexExpr = IndexExpression(left)
    let index = visitor.visitExpression(indexExpr.index).numberValue
    let arr = visitor.visitExpression(indexExpr.value)
    
    let indexValue = validIndex(index, arr.arrayLength)

    case arr.kind:
    of typeArray:
      arr.arrayData[][indexValue] = value
    of typeVec:
      arr.vecValue.values[indexValue] = value
    else:
      raise newException(ValueError, "not an array")

  elif left of DerefExpression:
    let ptrValue = visitor.visitExpression(DerefExpression(left).value)
    if ptrValue.kind == typeNul:
      raise newError(errSIGSEGV)
    ptrValue.ptrValue[] = value

  else:
    warn("AssignmentStatement unknown left")

method visitBranchingStatement*(visitor: InterpreterVisitor, node: BranchingStatement): auto =
  visitor.pushScope()
  
  try:
    if visitor.visitExpression(node.condition).boolValue:
      visitor.visitStatement(node.ifBlock)
      return
    
    for el in node.elifBlocks:
      if visitor.visitExpression(el.cond).boolValue:
        visitor.visitStatement(el.elifBlock)
        return
    
    if node.elseBlock != nil:
      visitor.visitStatement(node.elseBlock)
  
  finally:
    visitor.popScope()

method visitBreakStatement*(visitor: InterpreterVisitor, node: BreakStatement): auto =
  raise newException(BreakException, "")

method visitContinueStatement*(visitor: InterpreterVisitor, node: ContinueStatement): auto =
  raise newException(ContinueException, "")

method visitWhileStatement*(visitor: InterpreterVisitor, node: WhileStatement): auto =
  visitor.pushScope()

  try:
    while visitor.visitExpression(node.condition).boolValue:
      try:
        visitor.visitStatement(node.whileBlock)
      except BreakException:
        break
      except ContinueException:
        continue
        
  finally:
    visitor.popScope()

method visitDefaultStatement*(visitor: InterpreterVisitor, node: DefaultStatement): auto =
  visitor.newSlot(node.name.lexeme, newDefaultValue(node.symbolType))

method visitFuncStatement*(visitor: InterpreterVisitor, node: FuncStatement): auto =
  visitor.newSlot(node.name.lexeme, newFuncValue(node.funcType, node.arguments, node.funcBlock))

proc visitReturnStatement*(visitor: InterpreterVisitor, node: ReturnStatement): auto =
  let returnValue = visitor.visitExpression(node.value)
  var e = newException(ReturnException, "")
  e.value = returnValue
  raise e

method visitForStatement*(visitor: InterpreterVisitor, node: ForStatement): auto =
  visitor.pushScope()

  let varType = (if node.value.returnType.eq typeArray: node.value.returnType.arrBase
    else: node.value.returnType.vecBase)

  visitor.newSlot(node.name.lexeme, newDefaultValue(varType))

  let value = visitor.visitExpression(node.value)

  try:
    for i in 0..<value.arrayLength:
      try:
        visitor.setSlot(node.name.lexeme, value.arrayValues[i])
        visitor.visitStatement(node.forBlock)
      except BreakException:
        break
      except ContinueException:
        continue
        
  finally:
    visitor.popScope()

method visitCallStatement*(visitor: InterpreterVisitor, node: CallStatement): auto =
  discard visitor.visitExpression(node.callExpression)

# SPECIALS

proc get*(self: SpecialExpression | SpecialStatement, key: string): Expression =
  for token, expr in self.namedArgs.pairs:
    if token.lexeme == key:
      return expr
  return newErrorExpression(self.token)

proc has*(self: SpecialExpression | SpecialStatement, key: string): bool =
  for token, _ in self.namedArgs.pairs:
    if token.lexeme == key:
      return true
  return false

# SPECIALS

method visitSpecialExpression*(visitor: InterpreterVisitor, node: SpecialExpression): Value {.base.} =
  case node.kind:
  of skNew:
    let expr = node.get("0")
    let value = visitor.visitExpression(expr)
    var ptrValue = new(Value)
    ptrValue[] = value
    return newPtrValue(ptrValue, value.valueType)
    
  of skVec:
    if node.has("@"):
      let typeExpr = node.get("0")
      let baseType = typeExpr.returnType.arrBase
      let length = typeExpr.returnType.length
      
      var values: seq[Value] = @[]
      for i in 0..<length:
        values.add(newDefaultValue(baseType))
      
      return newVecValue(values, baseType)
    else:
      let expr = node.get("0")
      let arrValue = visitor.visitExpression(expr)
      
      var values: seq[Value] = @[]
      for val in arrValue.arrayValues:
        values.add(val)
      
      return newVecValue(values, arrValue.valueType.arrBase)
    
  of skLen:
    let expr = node.get("0")
    let arrValue = visitor.visitExpression(expr)
    return newInt64Value(int64(arrValue.arrayLength))

  of skFmt:
    var buffer: seq[Value] = @[]
    var sep = ""
    var repr = false

    if node.has("sep"):
      sep = visitor.visitExpression(node.get("sep")).stringValue

    if node.has("repr"):
      repr = visitor.visitExpression(node.get("repr")).boolValue

    for key, expr in node.namedArgs.pairs:
      if key.kind == tkNumber:
        if key.lexeme != "0":
          for ch in sep:
            buffer.add newCharValue(ch)

        var formatted = $visitor.visitExpression(expr)

        if repr:
          formatted = escapeString(formatted)

        for ch in formatted:
          if ch == '\0': break
          buffer.add newCharValue(ch)

    return newVecValue(buffer, getCharType())

  of skTake:
    let expr = node.get("0")
    let lengthNode = node.get("length")
    let length = visitor.visitExpression(lengthNode).numberValue
    
    let vecValue = visitor.visitExpression(expr)
    
    if vecValue.vecValue.length > length:
      raise newError(errArrayLengthMismatch, "dynamic array length " & $vecValue.vecValue.length & 
        " does not fit in static array length " & $length)
    
    var values: seq[Value] = @[]
    for i in 0..<length:
      if i < vecValue.vecValue.length:
        values.add(vecValue.vecValue.values[i])
      else:
        values.add(newDefaultValue(node.returnType.arrBase))
    
    return newStaticArrayValue(values, node.returnType, length)

  of skTakeof:
    let typ = node.get("0")
    let expr = node.get("1")
    let lengthNode = node.get("length")
    let length = visitor.visitExpression(lengthNode).numberValue
    
    let vecValue = visitor.visitExpression(expr)
    
    if vecValue.vecValue.length > length:
      raise newError(errArrayLengthMismatch, "dynamic array length " & $vecValue.vecValue.length & 
        " does not fit in static array length " & $length)
    
    var values: seq[Value] = @[]
    for i in 0..<length:
      if i < vecValue.vecValue.length:
        values.add(vecValue.vecValue.values[i])
      else:
        values.add(newDefaultValue(typ.returnType.arrBase))
    
    return newStaticArrayValue(values, node.returnType, length)

  of skJoin:
    let arr = visitor.visitExpression(node.get("0")).arrayValues
    let sep = visitor.visitExpression(node.get("1")).stringValue

    var buffer: seq[Value] = @[]

    var first = true
    for el in arr:
      if not first:
        for ch in sep:
          buffer.add newCharValue(ch)
      else:
        first = false

      for ch in $el:
        if ch == '\0': break
        buffer.add newCharValue(ch)

    return newVecValue(buffer, getCharType())

  of skRead:
    var buffer: seq[Value] = @[]

    for ch in stdin.readLine():
      buffer.add newCharValue(ch)

    buffer.add newCharValue('\0')

    return newVecValue(buffer, getCharType())
    
  else:
    warn("Unhandled special expression: ", node.kind)
    return newDefaultValue(getUndefinedType())

method visitSpecialStatement*(visitor: InterpreterVisitor, node: SpecialStatement): auto =
  case node.kind:
  of skPrint:
    var value = visitor.visitExpression(node.get("0"))
    var str = value.stringValue

    var term = "\n"
    if node.has("term"): 
      term = visitor.visitExpression(node.get("term")).stringValue

    stdout.write(str & term)

    if node.has("free") and visitor.visitExpression(node.get("free")).boolValue: 
      value.vecValue = nil
      
  of skFree:
    let expr = node.get("0")
    var value = visitor.visitExpression(expr)
    
    if value.kind == typePtr:
      value.ptrValue = nil
    elif value.kind == typeVec:
      value.vecValue = nil
      
  of skAssert:
    let cond = node.get("0")
    let condValue = visitor.visitExpression(cond)
    
    if not condValue.boolValue:
      if node.has("1"):
        let msg = node.get("1")
        let msgValue = visitor.visitExpression(msg)
        var str = ""
        for ch in msgValue.arrayValues:
          str.add(ch.charValue)
        raise newError(errAssert, str)
      else:
        raise newError(errAssert, "Assertion failed")

  of skResize:
    let value = visitor.visitExpression(node.get("0"))
    let size = visitor.visitExpression(node.get("1")).int64Value

    value.vecValue.values.setLen(size)
    for i in value.vecValue.length..<size:
      value.vecValue.values[i] = newDefaultValue(value.valueType.vecBase)
    value.vecValue.length = size
      
  else:
    warn("Unhandled special statement: ", node.kind)

# GENERAL

method visitExpression*(visitor: InterpreterVisitor, node: Expression): Value {.base.} =
  if node of ErrorExpression: discard
  elif node of TypeExpression: discard
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
  elif node of TupleExpression:
    return visitor.visitTupleExpression(TupleExpression(node))
  elif node of FieldExpression:
    return visitor.visitFieldExpression(FieldExpression(node))
  elif node of CallExpression:
    return visitor.visitCallExpression(CallExpression(node))
  else:
    warn "[InterpreterVisitor] WARNING: unhandled expression"

method visitStatement*(visitor: InterpreterVisitor, node: Statement) =
  if node of ErrorStatement: discard
  elif node of DeclarationStatement:
    visitor.visitDeclarationStatement(DeclarationStatement(node))
  elif node of BlockStatement:
    visitor.visitBlockStatement(BlockStatement(node))
  elif node of AssignmentStatement:
    visitor.visitAssignmentStatement(AssignmentStatement(node))
  elif node of BranchingStatement:
    visitor.visitBranchingStatement(BranchingStatement(node))
  elif node of SpecialStatement:
    visitor.visitSpecialStatement(SpecialStatement(node))
  elif node of BreakStatement:
    visitor.visitBreakStatement(BreakStatement(node))
  elif node of ContinueStatement:
    visitor.visitContinueStatement(ContinueStatement(node))
  elif node of WhileStatement:
    visitor.visitWhileStatement(WhileStatement(node))
  elif node of DefaultStatement:
    visitor.visitDefaultStatement(DefaultStatement(node))
  elif node of FuncStatement:
    visitor.visitFuncStatement(FuncStatement(node))
  elif node of ReturnStatement:
    visitor.visitReturnStatement(ReturnStatement(node))
  elif node of ForStatement:
    visitor.visitForStatement(ForStatement(node))
  elif node of CallStatement:
    visitor.visitCallStatement(CallStatement(node))
  else:
    warn "[InterpreterVisitor] WARNING: unhandled statement"