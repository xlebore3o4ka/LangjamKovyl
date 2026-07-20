package lexer

import "testing"

func TestBasicTokens(t *testing.T) {
	input := `let x = 10`
	lex := New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	expected := []TokenType{KW_LET, IDENT, ASSIGN, INT_LIT, EOF}
	if len(tokens) != len(expected) {
		t.Fatalf("expected %d tokens, got %d", len(expected), len(tokens))
	}
	for i, tt := range expected {
		if tokens[i].Type != tt {
			t.Errorf("token %d: expected %v, got %v", i, tt, tokens[i].Type)
		}
	}
}

func TestStringLiteral(t *testing.T) {
	input := `"hello world"`
	lex := New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if tokens[0].Type != STRING_LIT {
		t.Errorf("expected STRING_LIT, got %v", tokens[0].Type)
	}
	if tokens[0].Literal != "hello world" {
		t.Errorf("expected 'hello world', got '%s'", tokens[0].Literal)
	}
}

func TestStringEscapes(t *testing.T) {
	input := `"line1\nline2\ttab"`
	lex := New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	expected := "line1\nline2\ttab"
	if tokens[0].Literal != expected {
		t.Errorf("expected '%s', got '%s'", expected, tokens[0].Literal)
	}
}

func TestUnterminatedString(t *testing.T) {
	input := `"hello`
	lex := New(input)
	_, err := lex.Tokenize()
	if err == nil {
		t.Error("expected error for unterminated string")
	}
}

func TestBlockComment(t *testing.T) {
	input := `let /* comment */ x = 1`
	lex := New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	// should be: let x = 1 EOF
	if len(tokens) != 5 {
		t.Fatalf("expected 5 tokens (let, x, =, 1, EOF), got %d: %v", len(tokens), tokens)
	}
}

func TestNestedBlockComment(t *testing.T) {
	input := `let /* a /* b */ c */ x = 1`
	lex := New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(tokens) != 5 {
		t.Fatalf("expected 5 tokens, got %d: %v", len(tokens), tokens)
	}
}

func TestOperators(t *testing.T) {
	input := `+ - * / ++ == != < > <= >=`
	lex := New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	expected := []TokenType{PLUS, MINUS, STAR, SLASH, PLUS_PLUS, EQ_EQ, NOT_EQ, LT, GT, LT_EQ, GT_EQ, EOF}
	if len(tokens) != len(expected) {
		t.Fatalf("expected %d tokens, got %d", len(expected), len(tokens))
	}
}

func TestKeywords(t *testing.T) {
	input := `fn if else while for in let mut return actor msg struct true false and or not`
	lex := New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	expected := []TokenType{
		KW_FN, KW_IF, KW_ELSE, KW_WHILE, KW_FOR, KW_IN,
		KW_LET, KW_MUT, KW_RETURN, KW_ACTOR, KW_MSG, KW_STRUCT,
		KW_TRUE, KW_FALSE, KW_AND, KW_OR, KW_NOT, EOF,
	}
	if len(tokens) != len(expected) {
		t.Fatalf("expected %d tokens, got %d", len(expected), len(tokens))
	}
	for i, tt := range expected {
		if tokens[i].Type != tt {
			t.Errorf("token %d: expected %v (%s), got %v (%s)", i, tt, tokenNames[tt], tokens[i].Type, tokens[i].Literal)
		}
	}
}

func TestFloatLiteral(t *testing.T) {
	input := `3.14`
	lex := New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if tokens[0].Type != FLOAT_LIT {
		t.Errorf("expected FLOAT_LIT, got %v", tokens[0].Type)
	}
	if tokens[0].Literal != "3.14" {
		t.Errorf("expected '3.14', got '%s'", tokens[0].Literal)
	}
}

func TestUnexpectedChar(t *testing.T) {
	input := `@`
	lex := New(input)
	_, err := lex.Tokenize()
	if err == nil {
		t.Error("expected error for unexpected character")
	}
}

func TestLineTracking(t *testing.T) {
	input := "let x = 1\nlet y = 2"
	lex := New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	// second let should be on line 2
	if tokens[4].Type != KW_LET {
		t.Fatalf("expected KW_LET at index 4, got %v", tokens[4].Type)
	}
	if tokens[4].Line != 2 {
		t.Errorf("expected line 2, got line %d", tokens[4].Line)
	}
}

func TestFullProgram(t *testing.T) {
	input := `
use std.io

fn main() {
    io.println("Hello, World!")
}
`
	lex := New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	
	if tokens[len(tokens)-1].Type != EOF {
		t.Error("expected EOF at end")
	}
}

// func TestMultiCharOperators(t *testing.T) {
//     // TODO: test that >= doesn't lex as > then =
//     input := `a >= b`
//     lex := New(input)
//     tokens, _ := lex.Tokenize()
//     if tokens[1].Type != GT_EQ {
//         t.Errorf("expected GT_EQ, got %v", tokens[1].Type)
//     }
// }
