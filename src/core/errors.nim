import tokens
import std/[logging]

type
  ErrorKind* = enum
    errSyntax, errExpression, errStatement, errExpectedSyntax, errCannotAssign
    errForbiddenLocation, errSize, errEmptyStaticArray, errUnknownSize

    errMismatchedBracket, errUnexpectedBracket, errUnclosedBracket, 
    errUnclosedString, errUnclosedChar, errEmptyCharLiteral

    errBinaryTypeMismatch, errUnaryTypeMismatch, errTypeMismatch, 
    errUnknownType, errCannotCast

    errRedeclaration, errUndeclaredSymbol, errStmtSpecial, errExprSpecial

    errUnknownPragma

    errUnexpectedArgument, errUnexpectedNamedArgument, errMissingArgument, errDuplicateArgument
    errArgumentsNumber

    errHaventField

    errUnreachableCode, errMissingReturn, errFuncNamedArguments, errUnusedReturn

  CompileError* = ref object
    kind*: ErrorKind
    file*: string
    line*: Positive
    col*: Positive
    pos*: Natural
    len*: Positive
    args*: seq[(string, string)]
    message*: string

var errors*: seq[CompileError] = @[]

proc message(kind: ErrorKind): string =
  case kind
    of errSyntax: "Invalid syntax"
    of errExpression: "Expected expression, got @0"
    of errStatement: "Expected statement, got @0"
    of errExpectedSyntax: "Expected @0, got @1"
    of errCannotAssign: "Cannot assign to this expression"
    of errForbiddenLocation: "This node is located in a forbidden place"
    of errSize: "@0 does not fit in type @1"
    of errUnknownSize: "Cannot deduce the size from the context"
    of errEmptyStaticArray: "Static array cannot be empty"
    of errMismatchedBracket: "Mismatched bracket"
    of errUnexpectedBracket: "Unexpected closing bracket"
    of errUnclosedBracket: "Unclosed bracket"
    of errUnclosedString: "Unclosed string literal"
    of errUnclosedChar: "Unclosed character literal"
    of errEmptyCharLiteral: "Empty character literal"
    of errBinaryTypeMismatch: "Type mismatch for binary operator '@0' (@1 @0 @2)"
    of errUnaryTypeMismatch: "Type mismatch for unary operator '@0' (@1)"
    of errTypeMismatch: "Type mismatch (expected @0, got @1)"
    of errUnknownType: "Unknown type"
    of errCannotCast: "Cannot cast from @0 to @1"
    of errRedeclaration: "Redeclaration of symbol '@0', originally declared at @1(@2:@3)"
    of errUndeclaredSymbol: "Undeclared symbol '@0'"
    of errStmtSpecial: "Unknown special statement"
    of errExprSpecial: "Unknown special expression"
    of errUnknownPragma: "Unknown pragma"
    of errUnexpectedNamedArgument: "Unexpected named argument '@0'"
    of errUnexpectedArgument: "Unexpected argument at position @0"
    of errMissingArgument: "Missing required argument '@0'"
    of errDuplicateArgument: "Duplicate argument: @0"
    of errArgumentsNumber: "Expected @0 arguments, got @1"
    of errHaventField: "@0 does not have field '@1'"
    of errUnreachableCode: "The code after the statement declared at @0(@1:@2) is unreachable"
    of errMissingReturn: "Function '@0' does not return a value on all control paths"
    of errFuncNamedArguments: "Named arguments are prohibited in function arguments"
    of errUnusedReturn: "function '@0' returns a value and it must be used"

proc newError*(
              kind: ErrorKind, file: string, line: Positive, col: Positive, 
              pos: Natural, len: Positive, 
              args: seq[(string, string)] = @[]) =
  var msg = kind.message()
  
  errors.add(CompileError(
    kind: kind, file: file, line: line, col: col,
    pos: pos, len: len, args: args, message: msg
  ))

  var logMsg = $kind & " at " & file & ":" & $line & ":" & $col
  
  if args.len > 0:
    logMsg.add(" [")
    for i, (key, value) in args:
      if i > 0: logMsg.add(", ")
      logMsg.add(key & " = \"" & value & '"')
    logMsg.add("]")

  error(logMsg)

proc newError*(
              kind: ErrorKind, token: Token, 
              args: seq[(string, string)] = @[]) {.inline.} =
  newError(kind, token.file, token.line, token.column, token.offset, token.lexeme.len, args)