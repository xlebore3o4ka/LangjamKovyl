import std/[unicode, tables]
import tokens, errors

type 
  Lexer* = object
    text: string
    file: string
    len: Natural
    line: Positive = 1
    column: Positive = 1
    pos: Natural = 0

    hasError*: bool = false

    peekedToken*: Token
    hasPeeked*: bool = false

    bracketStack: seq[Token]
    lastIsEOS: bool = false

  RollbackData* = object
    pos*: Natural
    line*: Positive
    column*: Positive
    hasPeeked*: bool
    peekedToken*: Token
    lastIsEOS*: bool
    bracketStack: seq[Token]

proc getRollbackData*(self: Lexer): RollbackData =
  RollbackData(
    pos: self.pos,
    line: self.line,
    column: self.column,
    hasPeeked: self.hasPeeked,
    peekedToken: self.peekedToken,
    lastIsEOS: self.lastIsEOS,
    bracketStack: self.bracketStack
  )

proc rollback*(self: var Lexer, data: RollbackData) =
  self.pos = data.pos
  self.line = data.line
  self.column = data.column
  self.hasPeeked = data.hasPeeked
  self.peekedToken = data.peekedToken
  self.lastIsEOS = data.lastIsEOS
  self.bracketStack = data.bracketStack

proc getEOFToken*(self: Lexer): Token =
  let lastPos = if self.len > 0: self.len - 1 else: 0
  let lastRune = self.text.runeAt(lastPos)
  
  var line = 1
  var column = 1
  var pos = 0
  
  while pos < lastPos:
    let r = self.text.runeAt(pos)
    if r == '\n'.Rune:
      line.inc
      column = 1
    else:
      column.inc
    pos += r.size
  
  if lastRune == '\n'.Rune:
    return tkEOF.newToken("\0", self.file, line + 1, 1, self.len)
  else:
    return tkEOF.newToken("\0", self.file, line, column + 1, self.len)

proc newLexer*(text: string, file: string): Lexer =
  Lexer(text: text, file: file, len: text.len.Natural, peekedToken: tkEOF.newToken("\0", file, 1, 1, 0),
    bracketStack: newSeq[Token]())

func peek(self: Lexer): Rune {.inline.} =
  if self.pos >= self.len: return Rune(0)
  self.text.runeAt(self.pos)

func advance(self: var Lexer) {.inline.} = 
  self.pos += self.text.runeAt(self.pos).size
  self.column.inc

func isDigit(c: Rune): bool {.inline.} = int(c) in 48..57

const operatorTokens = {
  "+": tkPlus,
  "-": tkMinus,
  "*": tkStar,
  "/": tkSlash,
  "%": tkPercent,
  "=": tkEqual,
  ":": tkColon,
  ",": tkComma,
  "->": tkArrow,
  "$": tkDollar,
  ".": tkDot,
  "@": tkAt,
  "!": tkNot,
  ">": tkGT,
  "<": tkLT,
  "==": tkEQ,
  "!=": tkNEQ,
  ">=": tkGTE,
  "<=": tkLTE,
  "#": tkHash,
  "#!": tkPragma
}.toTable

const openBracketTokens = {
  '('.Rune: tkLParen,
  '['.Rune: tkLBracket,
  '{'.Rune: tkLBrace
}.toTable

const closeBracketTokens = {
  ')'.Rune: tkRParen,
  ']'.Rune: tkRBracket,
  '}'.Rune: tkRBrace
}.toTable

const pairBracketTokens = {
  tkLParen: ')'.Rune,
  tkRParen: '('.Rune,
  tkLBracket: ']'.Rune,
  tkRBracket: '['.Rune,
  tkLBrace: '}'.Rune,
  tkRBrace: '{'.Rune
}.toTable

const keywordsTokens = {
  "int": tkInt64, "int64": tkInt64, "int32": tkInt32, "int16": tkInt16, "int8": tkInt8,
  "uint": tkUint64, "uint64": tkUint64, "uint32": tkUint32, "uint16": tkUint16, "uint8": tkUint8,
  "bool": tkBool,
  "char": tkChar,
  "string": tkString,
  "nul": tkNul,
  "true": tkTrue,
  "false": tkFalse,
  "and": tkAnd,
  "or": tkOr,
  "do": tkDo,
  "end": tkEnd,
  "if": tkIf,
  "elif": tkElif,
  "else": tkElse,
  "while": tkWhile,
  "break": tkBreak,
  "continue": tkContinue,
  "func": tkFunc,
  "return": tkReturn,
  "for": tkFor,
  "pub": tkPub
}.toTable

proc newError(self: var Lexer, kind: ErrorKind, file: string, line, column, pos, len: int, 
              args: seq[(string, string)] = @[]) =
  self.hasError = true
  newError(kind, file, line, column, pos, len, args)

proc nextToken*(self: var Lexer): Token =
  if self.hasPeeked:
    self.hasPeeked = false
    return self.peekedToken
  while self.peek == ' '.Rune or self.peek == '\t'.Rune: self.advance
  let c = self.peek
  if c == Rune(0): 
    if self.bracketStack.len > 0:
      let last = self.bracketStack.pop()
      self.newError(errUnclosedBracket, last.file, last.line, last.column, last.offset, 1)
    result = tkEOF.newToken("\0", self.file, self.line, self.column, self.pos)
  elif c == '\n'.Rune: 
    while self.peek == '\n'.Rune:
      self.line.inc
      self.advance
    self.column = 1
    result = self.nextToken
  elif c == ';'.Rune:
    self.advance
    if self.lastIsEOS:
      result = self.nextToken()
    else:
      result = tkEOS.newToken(";", self.file, self.line, self.column, self.pos)
  elif c.isDigit:
    let column = self.column
    var start = self.pos
    var num = ""
    while self.peek.isDigit:
      num &= $self.peek
      self.advance()
    result = tkNumber.newToken(num, self.file, self.line, column, start)
  elif $c in operatorTokens:
    let column = self.column
    let pos = self.pos
    self.advance()
    var op = $c

    if $c & $self.peek() in operatorTokens:
      op &= self.peek()
      self.advance()

    let kind = operatorTokens[$op]

    if kind == tkHash or kind == tkPragma and pos == 0:
      while self.peek() != '\n'.Rune and self.peek() != '\0'.Rune: 
        self.advance()
      result = self.nextToken()
    else:
      result = kind.newToken(op, self.file, self.line, column, pos)
  elif c in openBracketTokens:
    let token = openBracketTokens[c].newToken($c, self.file, self.line, self.column, self.pos)
    self.bracketStack.add(token)
    result = token
    self.advance()
  elif c in closeBracketTokens:
    result = closeBracketTokens[c].newToken($c, self.file, self.line, self.column, self.pos)
    if self.bracketStack.len > 0:
      let last = self.bracketStack[^1]
      if pairBracketTokens[last.kind] == c:
        discard self.bracketStack.pop()
      else:
        self.newError(errMismatchedBracket, self.file, self.line, self.column, self.pos, 1)
        result = tkInvalid.newToken($c, self.file, self.line, self.column, self.pos)
    else:
      self.newError(errUnexpectedBracket, self.file, self.line, self.column, self.pos, 1)
      result = tkInvalid.newToken($c, self.file, self.line, self.column, self.pos)
    self.advance()
  elif c.isAlpha or c == '_'.Rune:
    let column = self.column
    var start = self.pos
    var ident = ""

    while (let p = self.peek; p.isDigit or p.isAlpha or p == '_'.Rune):
      ident &= $p
      self.advance()

    if ident in keywordsTokens:
      result = keywordsTokens[ident].newToken(ident, self.file, self.line, column, start)
    else:
      result = tkIdentifier.newToken(ident, self.file, self.line, column, start)
  elif c == '"'.Rune:
    self.advance()
    
    let column = self.column
    var start = self.pos
    var strbuffer = ""

    while self.peek != '"'.Rune and self.peek != '\0'.Rune and self.peek != '\n'.Rune:
      if self.peek == '\\'.Rune:
        self.advance()
        case self.peek
        of 'n'.Rune: strbuffer.add('\n')
        of '0'.Rune: strbuffer.add('\0')
        of 'r'.Rune: strbuffer.add('\r')
        of 't'.Rune: strbuffer.add('\t')
        of '"'.Rune: strbuffer.add('"')
        of '\\'.Rune: strbuffer.add('\\')
        else: strbuffer.add('\\'); strbuffer.add($self.peek)
        self.advance()
      else:
        strbuffer.add($self.peek)
        self.advance()

    if self.peek == '\0'.Rune or self.peek == '\n'.Rune:
      result = tkInvalid.newToken(strbuffer, self.file, self.line, column, start)
      self.newError(errUnclosedString, self.file, self.line, column - 1, start - 1, strbuffer.len + 1)
    else:
      self.advance()
      result = tkStringLiteral.newToken(strbuffer, self.file, self.line, column, start)
  elif c == '\''.Rune:
    self.advance()
    let column = self.column
    let start = self.pos
    
    if self.peek == '\''.Rune:
      self.newError(errEmptyCharLiteral, self.file, self.line, self.column, self.pos, 1)
      result = tkInvalid.newToken("", self.file, self.line, column, start)
      self.advance()
      return result
    
    var ch = $self.peek
    self.advance()
    if ch == "\\":
      case self.peek:
      of 'n'.Rune: ch = "\n"
      of '0'.Rune: ch = "\0"
      of 'r'.Rune: ch = "\r"
      of 't'.Rune: ch = "\t"
      of '\''.Rune: ch = "'"
      of '\\'.Rune: ch = "\\"
      else: discard
      self.advance()

    if self.peek != '\''.Rune:
      self.newError(errUnclosedChar, self.file, self.line, self.column, self.pos, 1)
      result = tkInvalid.newToken(ch, self.file, self.line, column, start)
    else:
      self.advance()
      result = tkCharLiteral.newToken(ch, self.file, self.line, column, start)
  else:
    self.newError(errSyntax, self.file, self.line, self.column, self.pos, 1)
    result = tkInvalid.newToken($c, self.file, self.line, self.column, self.pos)
    self.advance()

  self.lastIsEOS = result.kind == tkEOS

proc peekToken*(self: var Lexer): Token =
  if not self.hasPeeked:
    self.peekedToken = self.nextToken()
    self.hasPeeked = true
  return self.peekedToken
