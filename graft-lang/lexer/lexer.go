package lexer

import (
	"fmt"
	"unicode"
)

type TokenType int

const (
	// Literals
	INT_LIT TokenType = iota
	FLOAT_LIT
	STRING_LIT
	IDENT

	// Keywords
	KW_ACTOR
	KW_AND
	KW_BREAK
	KW_CONTINUE
	KW_ELSE
	KW_FALSE
	KW_FN
	KW_FOR
	KW_IF
	KW_IN
	KW_LET
	KW_LOOP
	KW_MOD
	KW_MSG
	KW_MUT
	KW_NOT
	KW_OR
	KW_RETURN
	KW_SPAWN
	KW_STATE
	KW_STRUCT
	KW_TRUE
	KW_USE
	KW_WHILE
	KW_MATCH
	KW_CASE
	KW_WILDCARD

	// Operators
	PLUS
	MINUS
	STAR
	SLASH
	PLUS_PLUS
	EQ_EQ
	NOT_EQ
	LT
	GT
	LT_EQ
	GT_EQ

	// Delimiters
	LPAREN
	RPAREN
	LBRACE
	RBRACE
	LBRACKET
	RBRACKET
	COMMA
	SEMICOLON
	DOT
	COLON
	ASSIGN

	// Special
	NEWLINE
	EOF
)

var keywords = map[string]TokenType{
	"actor":    KW_ACTOR,
	"and":      KW_AND,
	"break":    KW_BREAK,
	"continue": KW_CONTINUE,
	"else":     KW_ELSE,
	"false":    KW_FALSE,
	"fn":       KW_FN,
	"for":      KW_FOR,
	"if":       KW_IF,
	"in":       KW_IN,
	"let":      KW_LET,
	"loop":     KW_LOOP,
	"mod":      KW_MOD,
	"msg":      KW_MSG,
	"mut":      KW_MUT,
	"not":      KW_NOT,
	"or":       KW_OR,
	"return":   KW_RETURN,
	"struct":   KW_STRUCT,
	"true":     KW_TRUE,
	"use":      KW_USE,
	"while":    KW_WHILE,
	"match":    KW_MATCH,
	"case":     KW_CASE,
	"_":        KW_WILDCARD,
}

var tokenNames = map[TokenType]string{
	INT_LIT:    "INT",
	FLOAT_LIT:  "FLOAT",
	STRING_LIT: "STRING",
	IDENT:      "IDENT",
	PLUS:       "+",
	MINUS:      "-",
	STAR:       "*",
	SLASH:      "/",
	PLUS_PLUS:  "++",
	EQ_EQ:      "==",
	NOT_EQ:     "!=",
	LT:         "<",
	GT:         ">",
	LT_EQ:      "<=",
	GT_EQ:      ">=",
	LPAREN:     "(",
	RPAREN:     ")",
	LBRACE:     "{",
	RBRACE:     "}",
	LBRACKET:   "[",
	RBRACKET:   "]",
	COMMA:      ",",
	SEMICOLON:  ";",
	DOT:        ".",
	COLON:      ":",
	ASSIGN:     "=",
	NEWLINE:    "NL",
	EOF:        "EOF",
}

type Token struct {
	Type    TokenType
	Literal string
	Line    int
	Col     int
}

func (t Token) String() string {
	if name, ok := tokenNames[t.Type]; ok {
		return fmt.Sprintf("%s(%q)", name, t.Literal)
	}
	return fmt.Sprintf("TOKEN(%d, %q)", t.Type, t.Literal)
}

type Lexer struct {
	input  []rune
	pos    int
	line   int
	col    int
	tokens []Token
}

func New(input string) *Lexer {
	return &Lexer{
		input: []rune(input),
		pos:   0,
		line:  1,
		col:   1,
	}
}

func (l *Lexer) peek() rune {
	if l.pos >= len(l.input) {
		return 0
	}
	return l.input[l.pos]
}

func (l *Lexer) peekNext() rune {
	if l.pos+1 >= len(l.input) {
		return 0
	}
	return l.input[l.pos+1]
}

func (l *Lexer) advance() rune {
	ch := l.input[l.pos]
	l.pos++
	if ch == '\n' {
		l.line++
		l.col = 1
	} else {
		l.col++
	}
	return ch
}

func (l *Lexer) addToken(typ TokenType, literal string) {
	l.tokens = append(l.tokens, Token{
		Type:    typ,
		Literal: literal,
		Line:    l.line,
		Col:     l.col,
	})
}

func (l *Lexer) skipWhitespace() {
	for l.pos < len(l.input) {
		ch := l.peek()
		if ch == ' ' || ch == '\t' || ch == '\r' {
			l.advance()
		} else if ch == '\n' {
			l.advance()
		} else {
			break
		}
	}
}

func (l *Lexer) skipLineComment() {
	for l.pos < len(l.input) && l.peek() != '\n' {
		l.advance()
	}
}

func (l *Lexer) skipBlockComment() {
	depth := 1
	l.advance() // skip second *
	for l.pos < len(l.input) && depth > 0 {
		ch := l.peek()
		if ch == '*' && l.peekNext() == '/' {
			depth--
			l.advance() // *
			l.advance() // /
		} else if ch == '/' && l.peekNext() == '*' {
			depth++
			l.advance() // /
			l.advance() // *
		} else {
			l.advance()
		}
	}
}

func (l *Lexer) readString() (string, error) {
	l.advance() // skip opening quote
	var chars []rune
	for l.pos < len(l.input) {
		ch := l.peek()
		if ch == '"' {
			l.advance() // skip closing quote
			return string(chars), nil
		}
		if ch == '\\' {
			l.advance()
			esc := l.peek()
			switch esc {
			case 'n':
				chars = append(chars, '\n')
			case 't':
				chars = append(chars, '\t')
			case '\\':
				chars = append(chars, '\\')
			case '"':
				chars = append(chars, '"')
			default:
				chars = append(chars, esc)
			}
		} else {
			chars = append(chars, ch)
		}
		l.advance()
	}
	return "", fmt.Errorf("unterminated string at line %d", l.line)
}

func (l *Lexer) readNumber() Token {
	start := l.pos
	line, col := l.line, l.col
	isFloat := false

	for l.pos < len(l.input) && unicode.IsDigit(l.peek()) {
		l.advance()
	}
	if l.peek() == '.' && l.peekNext() != '.' {
		isFloat = true
		l.advance() // skip dot
		for l.pos < len(l.input) && unicode.IsDigit(l.peek()) {
			l.advance()
		}
	}

	lit := string(l.input[start:l.pos])
	tt := INT_LIT
	if isFloat {
		tt = FLOAT_LIT
	}
	return Token{Type: tt, Literal: lit, Line: line, Col: col}
}

func (l *Lexer) readIdent() Token {
	start := l.pos
	line, col := l.line, l.col

	for l.pos < len(l.input) && (unicode.IsLetter(l.peek()) || unicode.IsDigit(l.peek()) || l.peek() == '_') {
		l.advance()
	}

	lit := string(l.input[start:l.pos])
	tt, isKw := keywords[lit]
	if !isKw {
		tt = IDENT
	}
	return Token{Type: tt, Literal: lit, Line: line, Col: col}
}

func (l *Lexer) Tokenize() ([]Token, error) {
	for l.pos < len(l.input) {
		l.skipWhitespace()
		if l.pos >= len(l.input) {
			break
		}

		ch := l.peek()
		line, col := l.line, l.col

		switch {
		case ch == '/' && l.peekNext() == '/':
			l.skipLineComment()
		case ch == '/' && l.peekNext() == '*':
			l.advance()
			l.advance()
			l.skipBlockComment()

		case ch == '"':
			s, err := l.readString()
			if err != nil {
				return nil, err
			}
			l.tokens = append(l.tokens, Token{Type: STRING_LIT, Literal: s, Line: line, Col: col})

		case unicode.IsDigit(ch):
			tok := l.readNumber()
			l.tokens = append(l.tokens, tok)

		case unicode.IsLetter(ch) || ch == '_':
			tok := l.readIdent()
			l.tokens = append(l.tokens, tok)

		case ch == '+' && l.peekNext() == '+':
			l.advance()
			l.advance()
			l.tokens = append(l.tokens, Token{Type: PLUS_PLUS, Literal: "++", Line: line, Col: col})
		case ch == '=' && l.peekNext() == '=':
			l.advance()
			l.advance()
			l.tokens = append(l.tokens, Token{Type: EQ_EQ, Literal: "==", Line: line, Col: col})
		case ch == '!' && l.peekNext() == '=':
			l.advance()
			l.advance()
			l.tokens = append(l.tokens, Token{Type: NOT_EQ, Literal: "!=", Line: line, Col: col})
		case ch == '<' && l.peekNext() == '=':
			l.advance()
			l.advance()
			l.tokens = append(l.tokens, Token{Type: LT_EQ, Literal: "<=", Line: line, Col: col})
		case ch == '>' && l.peekNext() == '=':
			l.advance()
			l.advance()
			l.tokens = append(l.tokens, Token{Type: GT_EQ, Literal: ">=", Line: line, Col: col})

		case ch == '+':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: PLUS, Literal: "+", Line: line, Col: col})
		case ch == '-':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: MINUS, Literal: "-", Line: line, Col: col})
		case ch == '*':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: STAR, Literal: "*", Line: line, Col: col})
		case ch == '/':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: SLASH, Literal: "/", Line: line, Col: col})
		case ch == '<':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: LT, Literal: "<", Line: line, Col: col})
		case ch == '>':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: GT, Literal: ">", Line: line, Col: col})
		case ch == '(':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: LPAREN, Literal: "(", Line: line, Col: col})
		case ch == ')':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: RPAREN, Literal: ")", Line: line, Col: col})
		case ch == '{':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: LBRACE, Literal: "{", Line: line, Col: col})
		case ch == '}':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: RBRACE, Literal: "}", Line: line, Col: col})
		case ch == '[':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: LBRACKET, Literal: "[", Line: line, Col: col})
		case ch == ']':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: RBRACKET, Literal: "]", Line: line, Col: col})
		case ch == ',':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: COMMA, Literal: ",", Line: line, Col: col})
		case ch == ';':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: SEMICOLON, Literal: ";", Line: line, Col: col})
		case ch == '.':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: DOT, Literal: ".", Line: line, Col: col})
		case ch == ':':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: COLON, Literal: ":", Line: line, Col: col})
		case ch == '=':
			l.advance()
			l.tokens = append(l.tokens, Token{Type: ASSIGN, Literal: "=", Line: line, Col: col})

		default:
			return nil, fmt.Errorf("unexpected character %q at line %d:%d", ch, line, col)
		}
	}

	l.tokens = append(l.tokens, Token{Type: EOF, Literal: "", Line: l.line, Col: l.col})
	return l.tokens, nil
}
