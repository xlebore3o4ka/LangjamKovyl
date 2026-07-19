package codegen

import (
	"graft/lexer"
	"graft/parser"
	"strings"
	"testing"
)

func genFromSrc(t *testing.T, src string) string {
	t.Helper()
	lex := lexer.New(src)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("lexer: %v", err)
	}
	prog, err := parser.New(tokens).Parse()
	if err != nil {
		t.Fatalf("parser: %v", err)
	}
	return Generate(prog, "test")
}

func TestGenHelloWorld(t *testing.T) {
	src := `
use std.io
fn main() {
    io.println("Hello!")
}`
	erl := genFromSrc(t, src)
	if !strings.Contains(erl, "-module(test).") {
		t.Error("missing module declaration")
	}
	if !strings.Contains(erl, "graft_print_item") {
		t.Error("missing print helper")
	}
}

func TestGenFnDecl(t *testing.T) {
	src := `fn add(a, b) { return a + b }`
	erl := genFromSrc(t, src)
	if !strings.Contains(erl, "add(") {
		t.Error("missing add function")
	}
	// the return should be a throw
	if !strings.Contains(erl, "throw({return") {
		t.Error("return not desugared to throw")
	}
}

func TestGenIf(t *testing.T) {
	src := `
fn main() {
    if true { let x = 1 } else { let y = 2 }
}`
	erl := genFromSrc(t, src)
	if !strings.Contains(erl, "case (") {
		t.Error("if not translated to case")
	}
}

// TODO: test actor generation
// func TestGenActor(t *testing.T) {
//     src := `
// actor Chat {
//     msg send(text) {
//         io.println(text)
//     }
// }`
//     erl := genFromSrc(t, src)
//     if !strings.Contains(erl, "chat_start()") {
//         t.Error("missing actor start function")
//     }
// }

func TestGenWhile(t *testing.T) {
	src := `
fn main() {
    let i = 0
    while i < 10 { i = i + 1 }
}`
	erl := genFromSrc(t, src)
	// while should generate a recursive function
	if !strings.Contains(erl, "graft_loop_") {
		t.Error("while not desugared to loop function")
	}
}

func TestGenStruct(t *testing.T) {
	src := `struct Point { x, y }`
	erl := genFromSrc(t, src)
	if !strings.Contains(erl, "new_point(") {
		t.Error("missing struct constructor")
	}
}

// func TestGenNestedIf(t *testing.T) {
//     src := `
// fn main() {
//     if true {
//         if false { let x = 1 }
//     }
// }`
//     erl := genFromSrc(t, src)
//     count := strings.Count(erl, "case (")
//     if count != 2 {
//         t.Errorf("expected 2 case expressions, got %d", count)
//     }
// }
