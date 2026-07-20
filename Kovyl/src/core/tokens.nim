type
  TokenKind* = enum
    tkNumber
    tkStringLiteral
    tkCharLiteral
    tkIdentifier
    tkNul

    tkPlus, tkMinus, tkStar, tkSlash, tkPercent
    tkEQ, tkNEQ, tkGT, tkLT, tkGTE, tkLTE
    tkEqual, tkNot
    tkColon, tkComma, tkArrow, tkDollar, tkDot, tkAt
    tkHash, tkPragma

    tkLParen, tkRParen
    tkLBracket, tkRBracket
    tkLBrace, tkRBrace

    tkAnd
    tkOr

    tkTrue
    tkFalse

    tkInt64, tkInt32, tkInt16, tkInt8
    tkUint64, tkUint32, tkUint16, tkUint8
    tkBool
    tkChar
    tkString

    tkDo, tkEnd
    tkIf, tkElif, tkElse
    tkWhile, tkFor, tkBreak, tkContinue
    tkFunc, tkReturn
    tkPub

    tkEOS
    tkEOF
    tkInvalid

  Token* = object
    kind*: TokenKind
    lexeme*: string
    file*: string
    line*: Positive = 1
    column*: Positive = 1
    offset*: Natural

func newToken*(
              kind: TokenKind, lexeme: string, file: string, 
              line: Positive, column: Positive, offset: Natural): Token =
  Token(kind: kind, lexeme: lexeme, file: file, line: line, column: column, offset: offset)

func newFrom*(
    token: Token,
    kind: TokenKind = token.kind,
    lexeme: string = token.lexeme,
    file: string = token.file,
    line: Positive = token.line,
    column: Positive = token.column,
    offset: Natural = token.offset
): Token =
  Token(kind: kind, lexeme: lexeme, file: file, line: line, column: column, offset: offset)

proc mean*(kind: TokenKind): string =
  case kind:
  of tkNumber:        return "number"
  of tkStringLiteral: return "string literal"
  of tkCharLiteral:   return "character literal"
  of tkIdentifier:    return "identifier"
  of tkNul:           return "nul"

  of tkPlus:          return "plus operator '+'"
  of tkMinus:         return "minus operator '-'"
  of tkStar:          return "star operator '*'"
  of tkSlash:         return "slash operator '/'"
  of tkPercent:       return "percent operator '%'"

  of tkEQ:            return "equals operator '=='"
  of tkNEQ:           return "not equals operator '!='"
  of tkGT:            return "greater operator '>'"
  of tkLT:            return "less operator '<'"
  of tkGTE:           return "greater or equal operator '>='"
  of tkLTE:           return "less or equal operator '<='"

  of tkEqual:         return "equal operator '='"
  of tkNot:           return "not operator '!'"
  of tkColon:         return "colon operator ':'"
  of tkComma:         return "comma operator ','"
  of tkArrow:         return "arrow operator '->'"
  of tkDollar:        return "dollar operator '$'"
  of tkDot:           return "dot operator '.'"
  of tkAt:           return "at operator '@'"
  of tkHash:          return "hash operator '#'"
  of tkPragma:        return "pragma operator '#!'"

  of tkLParen:        return "left parenthesis '('"
  of tkRParen:        return "right parenthesis ')'"
  of tkLBracket:      return "left bracket '['"
  of tkRBracket:      return "right bracket ']'"
  of tkLBrace:        return "left brace '{'"
  of tkRBrace:        return "right brace '}'"

  of tkAnd:           return "and operator 'and'"
  of tkOr:            return "or operator 'or'"

  of tkInt64:         return "int64 type"
  of tkInt32:         return "int32 type"
  of tkInt16:         return "int16 type"
  of tkInt8:          return "int8 type"
  of tkUint64:        return "uint64 type"
  of tkUint32:        return "uint32 type"
  of tkUint16:        return "uint16 type"
  of tkUint8:         return "uint8 type"
  of tkBool:          return "bool type"
  of tkChar:          return "char type"
  of tkString:        return "string type"

  of tkTrue:          return "true literal"
  of tkFalse:         return "false literal"

  of tkEOS:           return "end of statement"
  of tkEOF:           return "end of file"

  of tkDo:            return "keyword do"
  of tkEnd:           return "keyword end"
  of tkIf:            return "keyword if"
  of tkElif:          return "keyword elif"
  of tkElse:          return "keyword else"
  of tkWhile:         return "keyword while"
  of tkFor:           return "keyword for"
  of tkBreak:         return "keyword break"
  of tkContinue:      return "keyword continue"
  of tkFunc:          return "keyword func"
  of tkReturn:        return "keyword return"
  of tkPub:           return "keyword pub"
  
  of tkInvalid:       return "invalid token"

proc mean*(token: Token): string =
  mean(token.kind)