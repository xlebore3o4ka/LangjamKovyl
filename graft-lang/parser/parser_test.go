package parser

import (
	"graft/lexer"
	"testing"
)

func mustParse(t *testing.T, input string) *Program {
	t.Helper()
	lex := lexer.New(input)
	tokens, err := lex.Tokenize()
	if err != nil {
		t.Fatalf("lexer error: %v", err)
	}
	prog, err := New(tokens).Parse()
	if err != nil {
		t.Fatalf("parser error: %v", err)
	}
	return prog
}

func TestParseFnDecl(t *testing.T) {
	prog := mustParse(t, `fn add(a, b) { return a + b }`)
	if len(prog.Tops) != 1 {
		t.Fatalf("expected 1 top-level, got %d", len(prog.Tops))
	}
	fn, ok := prog.Tops[0].(*FnDecl)
	if !ok {
		t.Fatal("expected FnDecl")
	}
	if fn.Name != "add" {
		t.Errorf("expected name 'add', got '%s'", fn.Name)
	}
	if len(fn.Params) != 2 {
		t.Errorf("expected 2 params, got %d", len(fn.Params))
	}
}

func TestParseLet(t *testing.T) {
	prog := mustParse(t, `fn main() { let x = 10 }`)
	fn := prog.Tops[0].(*FnDecl)
	if len(fn.Body.Stmts) != 1 {
		t.Fatalf("expected 1 stmt, got %d", len(fn.Body.Stmts))
	}
	let, ok := fn.Body.Stmts[0].(*LetStmt)
	if !ok {
		t.Fatalf("expected LetStmt, got %T", fn.Body.Stmts[0])
	}
	if let.Name != "x" {
		t.Errorf("expected 'x', got '%s'", let.Name)
	}
}

func TestParseIfElse(t *testing.T) {
	prog := mustParse(t, `
fn main() {
    if true { let x = 1 } else { let y = 2 }
}`)
	fn := prog.Tops[0].(*FnDecl)
	ifStmt, ok := fn.Body.Stmts[0].(*IfStmt)
	if !ok {
		t.Fatalf("expected IfStmt, got %T", fn.Body.Stmts[0])
	}
	if ifStmt.Else == nil {
		t.Error("expected else branch")
	}
}

func TestParseWhile(t *testing.T) {
	prog := mustParse(t, `
fn main() {
    let i = 0
    while i < 10 { i = i + 1 }
}`)
	fn := prog.Tops[0].(*FnDecl)
	if len(fn.Body.Stmts) != 2 {
		t.Fatalf("expected 2 stmts, got %d", len(fn.Body.Stmts))
	}
	_, ok := fn.Body.Stmts[1].(*WhileStmt)
	if !ok {
		t.Fatalf("expected WhileStmt, got %T", fn.Body.Stmts[1])
	}
}

func TestParseFor(t *testing.T) {
	prog := mustParse(t, `
fn main() {
    let items = [1, 2, 3]
    for item in items { let x = item }
}`)
	fn := prog.Tops[0].(*FnDecl)
	forStmt, ok := fn.Body.Stmts[1].(*ForStmt)
	if !ok {
		t.Fatalf("expected ForStmt, got %T", fn.Body.Stmts[1])
	}
	if forStmt.Var != "item" {
		t.Errorf("expected var 'item', got '%s'", forStmt.Var)
	}
}

func TestParseActor(t *testing.T) {
	prog := mustParse(t, `
actor Chat {
    msg send(name, text) {
        io.println(name)
    }
}`)
	if len(prog.Tops) != 1 {
		t.Fatalf("expected 1 top-level, got %d", len(prog.Tops))
	}
	actor, ok := prog.Tops[0].(*ActorDecl)
	if !ok {
		t.Fatal("expected ActorDecl")
	}
	if actor.Name != "Chat" {
		t.Errorf("expected 'Chat', got '%s'", actor.Name)
	}
	if len(actor.Msgs) != 1 {
		t.Fatalf("expected 1 msg, got %d", len(actor.Msgs))
	}
	if actor.Msgs[0].Name != "send" {
		t.Errorf("expected msg 'send', got '%s'", actor.Msgs[0].Name)
	}
}

func TestParseStruct(t *testing.T) {
	prog := mustParse(t, `struct Point { x, y }`)
	if len(prog.Tops) != 1 {
		t.Fatalf("expected 1 top-level, got %d", len(prog.Tops))
	}
	strct, ok := prog.Tops[0].(*StructDecl)
	if !ok {
		t.Fatal("expected StructDecl")
	}
	if strct.Name != "Point" {
		t.Errorf("expected 'Point', got '%s'", strct.Name)
	}
	if len(strct.Fields) != 2 {
		t.Errorf("expected 2 fields, got %d", len(strct.Fields))
	}
}

func TestParseUse(t *testing.T) {
	prog := mustParse(t, `use std.io`)
	if len(prog.Uses) != 1 {
		t.Fatalf("expected 1 use, got %d", len(prog.Uses))
	}
	if prog.Uses[0].Module != "std.io" {
		t.Errorf("expected 'std.io', got '%s'", prog.Uses[0].Module)
	}
}

func TestParseExpressions(t *testing.T) {
	prog := mustParse(t, `fn main() { let x = 1 + 2 * 3 }`)
	fn := prog.Tops[0].(*FnDecl)
	let := fn.Body.Stmts[0].(*LetStmt)
	bin, ok := let.Init.(*BinExpr)
	if !ok {
		t.Fatalf("expected BinExpr, got %T", let.Init)
	}
	if bin.Op != "+" {
		t.Errorf("expected '+', got '%s'", bin.Op)
	}
	// right side should be 2 * 3
	right, ok := bin.Right.(*BinExpr)
	if !ok {
		t.Fatalf("expected BinExpr on right, got %T", bin.Right)
	}
	if right.Op != "*" {
		t.Errorf("expected '*', got '%s'", right.Op)
	}
}

func TestParseMapLit(t *testing.T) {
	prog := mustParse(t, `fn main() { let m = {"a": 1, "b": 2} }`)
	fn := prog.Tops[0].(*FnDecl)
	let := fn.Body.Stmts[0].(*LetStmt)
 mMap, ok := let.Init.(*MapLit)
	if !ok {
		t.Fatalf("expected MapLit, got %T", let.Init)
	}
	if len(mMap.Keys) != 2 {
		t.Errorf("expected 2 keys, got %d", len(mMap.Keys))
	}
}

func TestParseChainedMethodCalls(t *testing.T) {
	prog := mustParse(t, `fn main() { let x = obj.foo(1).bar(2) }`)
	fn := prog.Tops[0].(*FnDecl)
	let := fn.Body.Stmts[0].(*LetStmt)
	call, ok := let.Init.(*CallExpr)
	if !ok {
		t.Fatalf("expected CallExpr, got %T", let.Init)
	}
	dot, ok := call.Func.(*DotExpr)
	if !ok {
		t.Fatalf("expected DotExpr, got %T", call.Func)
	}
	if dot.Field != "bar" {
		t.Errorf("expected 'bar', got '%s'", dot.Field)
	}
}

// TODO: test error cases
