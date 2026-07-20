import std/[strutils, tables, sequtils, sugar]

type
  TypeKind* = enum
    typeUndefined
    typeInt64, typeInt32, typeInt16, typeInt8
    typeUint64, typeUint32, typeUint16, typeUint8

    typeBool
    typeChar
    typeArray

    typePtr
    typeVec 
    typeNul

    typeTuple
    typeFunc

  Type* = ref object
    case kind*: TypeKind
    of typePtr: ptrBase*: Type
    of typeVec: vecBase*: Type
    of typeArray:
      arrBase*: Type
      length*: Natural
    of typeTuple:
      elements*: OrderedTable[string, Type]
    of typeFunc:
      arguments*: OrderedTable[string, Type]
      returnType*: Type
    else: discard

let
  undefinedType* = Type(kind: typeUndefined)
  int64Type* = Type(kind: typeInt64)
  int32Type* = Type(kind: typeInt32)
  int16Type* = Type(kind: typeInt16)
  int8Type* = Type(kind: typeInt8)
  uint64Type* = Type(kind: typeUint64)
  uint32Type* = Type(kind: typeUint32)
  uint16Type* = Type(kind: typeUint16)
  uint8Type* = Type(kind: typeUint8)
  boolType* = Type(kind: typeBool)
  charType* = Type(kind: typeChar)

  nulType* = Type(kind: typeNul)

proc `$`*(k: TypeKind): string =
  case k
  of typeUndefined: "undefined"
  of typeInt64: "int64"
  of typeInt32: "int32"
  of typeInt16: "int16"
  of typeInt8: "int8"
  of typeUint64: "uint64"
  of typeUint32: "uint32"
  of typeUint16: "uint16"
  of typeUint8: "uint8"
  of typeBool: "bool"
  of typePtr: "T*"
  of typeChar: "char"
  of typeVec: "T@"
  of typeArray: "T[]"
  of typeNul: "nul"
  of typeTuple: "(T, ...)"
  of typeFunc: "(T, ...) -> T"

proc isValidUint*[T: SomeUnsignedInt](s: string): bool =
  try:
    let v = parseUInt(s)
    return v <= high(T)
  except ValueError:
    return false

proc `$`*(t: Type): string =
  if t == nil: return "nilType"
  case t.kind
  of typePtr: $t.ptrBase & "*"
  of typeVec: $t.vecBase & "[*]"
  of typeArray: $t.arrBase & "[" & (if t.length == 0: "" 
    else: $t.length) & "]" 
  of typeTuple: 
    let parts = collect:
      for k, v in t.elements:
        if isValidUint[uint64](k): $v else: $v & " " & k
    "(" & parts.join(", ") & ")"
  of typeFunc:
    let argsStr = t.arguments.values.toSeq
      .mapIt($it)
      .join(", ")
    return "(" & argsStr & ") -> " & (if t.returnType.kind != typeUndefined: $t.returnType else: "()")
  else: return $t.kind

var ptrTypes*: seq[Type] = @[]
var vecTypes*: seq[Type] = @[]
var arrayTypes*: seq[Type] = @[]
var tupleTypes*: seq[Type] = @[]
var funcTypes*: seq[Type] = @[]

proc getUndefinedType*(): Type {.inline.} = undefinedType
proc getInt64Type*(): Type {.inline.} = int64Type
proc getInt32Type*(): Type {.inline.} = int32Type
proc getInt16Type*(): Type {.inline.} = int16Type
proc getInt8Type*(): Type {.inline.} = int8Type
proc getUint64Type*(): Type {.inline.} = uint64Type
proc getUint32Type*(): Type {.inline.} = uint32Type
proc getUint16Type*(): Type {.inline.} = uint16Type
proc getUint8Type*(): Type {.inline.} = uint8Type
proc getBoolType*(): Type {.inline.} = boolType
proc getCharType*(): Type {.inline.} = charType

proc getNulType*(): Type {.inline.} = nulType

proc eq*(a: TypeKind, b: TypeKind): bool {.inline.} =
  return a == b

proc eq*(a: TypeKind, b: Type): bool {.inline.} =
  return a == b.kind

proc eq*(a: Type, b: TypeKind): bool {.inline.} =
  return a.kind == b

proc eq*(a: Type, b: Type): bool =
  if a.eq(typeArray) and a.length == 0:
    return b.eq(typeArray) and a.arrBase.eq b.arrBase
  if b.eq(typeArray) and b.length == 0:
    return a.eq(typeArray) and a.arrBase.eq b.arrBase
  if a.eq(typeTuple):
    return b.eq(typeTuple) and a.elements == b.elements
  if a.eq(typeFunc) and b.eq(typeFunc):
    if not a.returnType.eq(b.returnType): return false
    if a.arguments.len != b.arguments.len: return false
    for key, val in a.arguments:
      if key notin b.arguments: return false
      if not val.eq(b.arguments[key]): return false
    return true

  return a == b

proc neq*(a: Type | TypeKind, b: Type | TypeKind): bool {.inline.} =
  return not (a.eq b)

proc getPtrType*(baseType: Type): Type =
  if baseType.kind == typeUndefined:
    return baseType

  for t in ptrTypes:
    if t.ptrBase.eq baseType:
      return t
  
  result = Type(kind: typePtr, ptrBase: baseType)
  ptrTypes.add(result)

proc getVecType*(baseType: Type): Type =
  if baseType.kind == typeUndefined:
    return baseType

  for t in vecTypes:
    if t.vecBase.eq baseType:
      return t
  
  result = Type(kind: typeVec, vecBase: baseType)
  vecTypes.add(result)

proc getArrayType*(baseType: Type, length: Natural): Type =
  if baseType.kind == typeUndefined:
    return baseType

  for t in arrayTypes:
    if t.arrBase.eq(baseType) and t.length == length:
      return t
  
  result = Type(kind: typeArray, arrBase: baseType, length: length)
  arrayTypes.add(result)

proc getTupleType*(elements: OrderedTable[string, Type]): Type =
  if elements.len == 0:
    return getUndefinedType()

  for t in tupleTypes:
    if t.elements == elements:
      return t

  result = Type(kind: typeTuple, elements: elements)
  tupleTypes.add(result)

proc getFuncType*(args: OrderedTable[string, Type], returnType: Type): Type =
  for t in funcTypes:
    if t.arguments == args and t.returnType == returnType:
      return t

  result = Type(kind: typeFunc, arguments: args, returnType: returnType)
  funcTypes.add(result)