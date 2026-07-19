package codegen

import (
	"fmt"
	"graft/parser"
	"strings"
)


type Generator struct {
	builder      *strings.Builder
	indent       int
	moduleName   string
	exports      []string
	functions    map[string]bool
	whileLoops   []string
	actorName    string
	actorVars    map[string]string
	actorNames   map[string]bool
	mutVars      map[string]string
	mutCounters  map[string]int
	currentState string
	structs      map[string]bool
	receiveCount int
}

func Generate(prog *parser.Program, moduleName string) string {
	g := &Generator{
		moduleName:  moduleName,
		builder:     &strings.Builder{},
		actorVars:   make(map[string]string),
		actorNames:  make(map[string]bool),
		mutVars:     make(map[string]string),
		mutCounters: make(map[string]int),
		structs:     make(map[string]bool),
		functions:   make(map[string]bool),
	}
	g.genProgram(prog)
	return g.builder.String()
}

func (g *Generator) w(format string, args ...interface{}) {
	g.builder.WriteString(fmt.Sprintf(format, args...))
}

func (g *Generator) ind() string {
	return strings.Repeat("    ", g.indent)
}

func (g *Generator) genProgram(prog *parser.Program) {
	g.w("-module(%s).\n", g.moduleName)

	for _, top := range prog.Tops {
		switch t := top.(type) {
		case *parser.FnDecl:
			g.exports = append(g.exports, fmt.Sprintf("%s/%d", strings.ToLower(t.Name), len(t.Params)))
			g.functions[strings.ToLower(t.Name)] = true
		case *parser.ActorDecl:
			g.exports = append(g.exports, fmt.Sprintf("%s_start/0", strings.ToLower(t.Name)))
		case *parser.StructDecl:
			g.exports = append(g.exports, fmt.Sprintf("new_%s/%d", strings.ToLower(t.Name), len(t.Fields)))
			g.structs[t.Name] = true
		}
	}

	for _, top := range prog.Tops {
		switch t := top.(type) {
		case *parser.ActorDecl:
			g.actorNames[t.Name] = true
		}
	}

	for _, top := range prog.Tops {
		switch t := top.(type) {
		case *parser.FnDecl:
			g.preScanActorVars(t.Body)
		}
	}

	g.w("-export([")
	for i, e := range g.exports {
		if i > 0 {
			g.w(", ")
		}
		g.w("%s", e)
	}
	g.w("]).\n\n")

	for _, top := range prog.Tops {
		switch t := top.(type) {
		case *parser.FnDecl:
			g.genFn(t)
		case *parser.ActorDecl:
			g.genActor(t)
		case *parser.StructDecl:
			g.genStruct(t)
		}
	}

	for _, loop := range g.whileLoops {
		g.w("%s\n", loop)
	}

	g.w("graft_print_item(V) -> case io_lib:printable_list(V) of true -> io:format(\"~s\", [V]); false -> io:format(\"~p\", [V]) end.\n\n")
}

func (g *Generator) preScanActorVars(block *parser.Block) {
	for _, stmt := range block.Stmts {
		switch s := stmt.(type) {
		case *parser.LetStmt:
			if call, ok := s.Init.(*parser.CallExpr); ok {
				if dot, ok := call.Func.(*parser.DotExpr); ok {
					if ident, ok := dot.Object.(*parser.Ident); ok {
						if dot.Field == "spawn" && g.actorNames[ident.Name] {
							g.actorVars[s.Name] = ident.Name
						}
					}
				}
			}
		case *parser.IfStmt:
			if s.Then != nil {
				g.preScanActorVars(s.Then)
			}
			if s.Else != nil {
				g.preScanActorVars(s.Else)
			}
			if s.ElseIf != nil {
				g.preScanActorVars(&parser.Block{Stmts: []parser.Statement{s.ElseIf}})
			}
		case *parser.WhileStmt:
			g.preScanActorVars(s.Body)
		case *parser.ForStmt:
			g.preScanActorVars(s.Body)
		}
	}
}

func (g *Generator) genFn(fn *parser.FnDecl) {
	params := make([]string, len(fn.Params))
	for i, p := range fn.Params {
		params[i] = erlVar(p)
	}
	g.w("%s(%s) ->\n", strings.ToLower(fn.Name), strings.Join(params, ", "))
	g.indent++
	g.w("%stry\n", g.ind())
	g.indent++
	g.genBlock(fn.Body, "")
	g.w("%scatch\n", g.ind())
	g.w("%sthrow:{return, Val} -> Val\n", g.ind())
	g.indent--
	g.w("%send%s\n", g.ind(), ".")
	g.indent--
	g.w("\n\n")
}

func (g *Generator) genBlock(block *parser.Block, trailing string) {
	n := len(block.Stmts)
	for i, stmt := range block.Stmts {
		isLast := i == n-1
		sep := ","
		if isLast {
			sep = trailing
		}
		g.genStmt(stmt, sep)
	}
}

func (g *Generator) genStmt(stmt parser.Statement, trailing string) {
	g.w("%s", g.ind())

	switch s := stmt.(type) {
	case *parser.LetStmt:
		if call, ok := s.Init.(*parser.CallExpr); ok {
			if dot, ok := call.Func.(*parser.DotExpr); ok {
				if ident, ok := dot.Object.(*parser.Ident); ok {
					if dot.Field == "spawn" {
						g.actorVars[s.Name] = ident.Name
					}
				}
			}
		}
		g.w("%s = ", erlVar(s.Name))
		g.genExpr(s.Init)
		g.w("%s\n", trailing)

	case *parser.MutAssign:
		erlName := erlVar(s.Name)
		g.mutVars[s.Name] = erlName
		g.mutCounters[s.Name] = 0
		g.w("%s = ", erlName)
		g.genExpr(s.Init)
		g.w("%s\n", trailing)

	case *parser.AssignStmt:
		if currentErlName, ok := g.mutVars[s.Name]; ok {
			// Mut variable reassignment: create new variable name
			g.mutCounters[s.Name]++
			nextName := fmt.Sprintf("%s%d", currentErlName, g.mutCounters[s.Name])
			g.w("%s = ", nextName)
			g.genExpr(s.Val)
			g.w("%s\n", trailing)
			g.mutVars[s.Name] = nextName
		} else {
			g.w("%s = ", erlVar(s.Name))
			g.genExpr(s.Val)
			g.w("%s\n", trailing)
		}

	case *parser.DotAssignStmt:
		g.w("%s = maps:put('%s', ", g.currentStateVar(), s.Field)
		g.genExpr(s.Val)
		g.w(", %s)%s\n", g.currentStateVar(), trailing)

	case *parser.ReturnStmt:
		if s.Val != nil {
			g.w("throw({return, ")
			g.genExpr(s.Val)
			g.w("})%s\n", trailing)
		} else {
			g.w("throw({return, ok})%s\n", trailing)
		}

	case *parser.IfStmt:
		g.genIf(s, trailing)

	case *parser.WhileStmt:
		g.genWhile(s, trailing)

	case *parser.ForStmt:
		g.genFor(s, trailing)

	case *parser.MatchStmt:
		g.genMatch(s, trailing)

	case *parser.BreakStmt:
		g.w("throw({break})%s\n", trailing)

	case *parser.ContinueStmt:
		g.w("throw({continue})%s\n", trailing)

	case *parser.ExprStmt:
		g.genExpr(s.Expr)
		g.w("%s\n", trailing)
	}
}

func (g *Generator) genIf(ifStmt *parser.IfStmt, trailing string) {
	g.w("case (")
	g.genExpr(ifStmt.Cond)
	g.w(") of\n")

	g.indent++
	g.w("%strue ->\n", g.ind())
	g.indent++
	g.genBlock(ifStmt.Then, "")
	g.indent--

	if ifStmt.ElseIf != nil {
		g.w("%s;\n", g.ind())
		g.w("%s_ ->\n", g.ind())
		g.indent++
		g.genIf(ifStmt.ElseIf, "")
		g.indent--
	} else if ifStmt.Else != nil {
		g.w("%s;\n", g.ind())
		g.w("%s_ ->\n", g.ind())
		g.indent++
		g.genBlock(ifStmt.Else, "")
		g.indent--
	} else {
		g.w("%s;\n", g.ind())
		g.w("%s_ -> ok\n", g.ind())
	}
	g.indent--
	g.w("%send%s\n", g.ind(), trailing)
}

func (g *Generator) genWhile(ws *parser.WhileStmt, trailing string) {
	refs := make(map[string]bool)
	g.exprRefs(ws.Cond, refs)
	for _, stmt := range ws.Body.Stmts {
		g.stmtRefs(stmt, refs)
	}
	defs := make(map[string]bool)
	for _, stmt := range ws.Body.Stmts {
		g.stmtDefs(stmt, defs)
	}
	modules := map[string]bool{"io": true, "list": true, "string": true, "time": true, "os": true, "net": true, "process": true, "map": true}
	var vars []string
	seen := make(map[string]bool)
	for name := range refs {
		if defs[name] || seen[name] || name == "state" || modules[name] || g.functions[strings.ToLower(name)] {
			continue
		}
		seen[name] = true
		vars = append(vars, erlVar(name))
	}
	label := fmt.Sprintf("graft_loop_%d", ws.Line)

	var buf strings.Builder
	oldBuilder := g.builder
	oldIndent := g.indent
	g.builder = &buf
	g.indent = 0

	g.w("%s(%s) ->\n", label, strings.Join(vars, ", "))
	g.indent++
	g.w("case (")
	g.genExpr(ws.Cond)
	g.w(") of\n")
	g.indent++
	g.w("true ->\n")
	g.indent++

	nextVars := make([]string, len(vars))
	copy(nextVars, vars)
	savedMutVars := make(map[string]string)
	for k, v := range g.mutVars {
		savedMutVars[k] = v
	}
	for _, stmt := range ws.Body.Stmts {
		switch s := stmt.(type) {
		case *parser.AssignStmt:
			nextName := erlVar(s.Name) + "1"
			g.w("%s%s = ", g.ind(), nextName)
			g.genExpr(s.Val)
			g.w(",\n")
			for i, v := range nextVars {
				if v == erlVar(s.Name) {
					nextVars[i] = nextName
				}
			}
		}
	}

	mutState := make(map[string]string)
	for k, v := range savedMutVars {
		mutState[k] = v
	}
	g.w("%sBreakFlag = try\n", g.ind())
	g.indent++

	emitted := false
	for _, stmt := range ws.Body.Stmts {
		switch s := stmt.(type) {
		case *parser.AssignStmt:
			nextName := erlVar(s.Name) + "1"
			mutState[s.Name] = nextName
			continue
		}
		if emitted {
			g.w("%s,\n", g.ind())
		}
		emitted = true
		g.w("%s", g.ind())
		
		for k, v := range mutState {
			g.mutVars[k] = v
		}
		switch s := stmt.(type) {
		case *parser.ExprStmt:
			g.genExpr(s.Expr)
		default:
			g.genStmt(stmt, "")
		}
	}
	if !emitted {
		g.w("%sfalse", g.ind())
	} else {
		g.w(",\n%sfalse", g.ind())
	}
	g.w("\n")
	g.indent--

	g.w("%scatch\n", g.ind())
	g.indent++
	g.w("%sthrow:{break} -> true;\n", g.ind())
	g.w("%sthrow:{continue} -> false\n", g.ind())
	g.indent--
	g.w("%send,\n", g.ind())

	g.w("%scase BreakFlag of\n", g.ind())
	g.indent++
	g.w("%strue -> ok;\n", g.ind())
	g.w("%sfalse -> %s(%s)\n", g.ind(), label, strings.Join(nextVars, ", "))
	g.indent--
	g.w("%send;\n", g.ind())
	g.w("%s_ -> ok\n", g.ind())
	g.indent--
	g.w("%send.\n\n", g.ind())

	loopCode := buf.String()
	g.builder = oldBuilder
	g.indent = oldIndent
	g.mutVars = savedMutVars
	g.whileLoops = append(g.whileLoops, loopCode)

	g.w("%s(%s)%s\n", label, strings.Join(vars, ", "), trailing)
}

func (g *Generator) collectWhileVars(block *parser.Block) []string {
	refs := make(map[string]bool)
	defs := make(map[string]bool)

	for _, stmt := range block.Stmts {
		g.stmtRefs(stmt, refs)
		g.stmtDefs(stmt, defs)
	}

	var result []string
	seen := make(map[string]bool)
	for name := range refs {
		if defs[name] || seen[name] {
			continue
		}
		if name == "state" {
			continue
		}
		seen[name] = true
		result = append(result, name)
	}
	return result
}

func (g *Generator) stmtRefs(stmt parser.Statement, refs map[string]bool) {
	switch s := stmt.(type) {
	case *parser.LetStmt:
		g.exprRefs(s.Init, refs)
	case *parser.MutAssign:
		g.exprRefs(s.Init, refs)
	case *parser.AssignStmt:
		g.exprRefs(s.Val, refs)
	case *parser.DotAssignStmt:
		g.exprRefs(s.Object, refs)
		g.exprRefs(s.Val, refs)
	case *parser.ReturnStmt:
		if s.Val != nil {
			g.exprRefs(s.Val, refs)
		}
	case *parser.IfStmt:
		g.exprRefs(s.Cond, refs)
		g.blockRefs(s.Then, refs)
		if s.Else != nil {
			g.blockRefs(s.Else, refs)
		}
		if s.ElseIf != nil {
			g.stmtRefs(s.ElseIf, refs)
		}
	case *parser.WhileStmt:
		g.exprRefs(s.Cond, refs)
		g.blockRefs(s.Body, refs)
	case *parser.ForStmt:
		g.exprRefs(s.Seq, refs)
		g.blockRefs(s.Body, refs)
	case *parser.MatchStmt:
		g.exprRefs(s.Target, refs)
		for _, c := range s.Cases {
			if c.Pattern != nil {
				g.exprRefs(c.Pattern, refs)
			}
			g.blockRefs(c.Body, refs)
		}
	case *parser.ExprStmt:
		g.exprRefs(s.Expr, refs)
	}
}

func (g *Generator) blockRefs(block *parser.Block, refs map[string]bool) {
	for _, stmt := range block.Stmts {
		g.stmtRefs(stmt, refs)
	}
}

func (g *Generator) stmtDefs(stmt parser.Statement, defs map[string]bool) {
	switch s := stmt.(type) {
	case *parser.LetStmt:
		defs[s.Name] = true
	case *parser.MutAssign:
		defs[s.Name] = true
	case *parser.IfStmt:
		if s.Then != nil {
			for _, st := range s.Then.Stmts {
				g.stmtDefs(st, defs)
			}
		}
		if s.Else != nil {
			for _, st := range s.Else.Stmts {
				g.stmtDefs(st, defs)
			}
		}
		if s.ElseIf != nil {
			g.stmtDefs(s.ElseIf, defs)
		}
	case *parser.WhileStmt:
		for _, st := range s.Body.Stmts {
			g.stmtDefs(st, defs)
		}
	case *parser.ForStmt:
		defs[s.Var] = true
		for _, st := range s.Body.Stmts {
			g.stmtDefs(st, defs)
		}
	case *parser.MatchStmt:
		for _, c := range s.Cases {
			for _, st := range c.Body.Stmts {
				g.stmtDefs(st, defs)
			}
		}
	}
}

func (g *Generator) exprRefs(expr parser.Expr, refs map[string]bool) {
	if expr == nil {
		return
	}
	switch e := expr.(type) {
	case *parser.Ident:
		refs[e.Name] = true
	case *parser.BinExpr:
		g.exprRefs(e.Left, refs)
		g.exprRefs(e.Right, refs)
	case *parser.UnaryExpr:
		g.exprRefs(e.Expr, refs)
	case *parser.CallExpr:
		g.exprRefs(e.Func, refs)
		for _, arg := range e.Args {
			g.exprRefs(arg, refs)
		}
	case *parser.DotExpr:
		g.exprRefs(e.Object, refs)
	case *parser.IndexExpr:
		g.exprRefs(e.Object, refs)
		g.exprRefs(e.Index, refs)
	case *parser.ListLit:
		for _, elem := range e.Elems {
			g.exprRefs(elem, refs)
		}
	case *parser.MapLit:
		for _, val := range e.Values {
			g.exprRefs(val, refs)
		}
	case *parser.FuncLit:
		for _, stmt := range e.Body.Stmts {
			g.stmtRefs(stmt, refs)
		}
	}
}

func (g *Generator) genFor(fs *parser.ForStmt, trailing string) {
	g.w("lists:foreach(fun(%s) ->\n", erlVar(fs.Var))
	g.indent++
	g.genBlock(fs.Body, "")
	g.indent--
	g.w("%send, ", g.ind())
	g.genExpr(fs.Seq)
	g.w(")%s\n", trailing)
}

func (g *Generator) genMatch(m *parser.MatchStmt, trailing string) {
	g.w("case ")
	g.genExpr(m.Target)
	g.w(" of\n")
	g.indent++

	for i, c := range m.Cases {
		g.w("%s", g.ind())
		if c.Pattern == nil {
			g.w("_")
		} else {
			g.genExpr(c.Pattern)
		}
		g.w(" ->\n")
		g.indent++
		g.genBlock(c.Body, "")
		g.indent--

		if i < len(m.Cases)-1 {
			g.w("%s;\n", g.ind())
		} else {
			g.w("\n")
		}
	}

	g.indent--
	g.w("%send%s\n", g.ind(), trailing)
}

func (g *Generator) genActor(actor *parser.ActorDecl) {
	oldActor := g.actorName
	g.actorNames[actor.Name] = true
	g.actorName = strings.ToLower(actor.Name)
	actorName := g.actorName

	g.w("%s_start() ->\n", actorName)
	g.indent++
	g.w("%sspawn(fun() -> %s_loop(#{}) end).\n\n", g.ind(), actorName)
	g.indent--

	g.w("%s_loop(State) ->\n", actorName)
	g.indent++
	g.w("%sreceive\n", g.ind())

	for i, msg := range actor.Msgs {
		g.w("%s{", g.ind())
		g.w("%s", msg.Name)
		for _, param := range msg.Params {
			g.w(", %s", erlVar(param))
		}
		g.w(", From} ->\n")

		g.indent++

		stateN := 0
		g.currentState = "State"

		for _, stmt := range msg.Body.Stmts {
			g.w("%s", g.ind())
			switch s := stmt.(type) {
			case *parser.DotAssignStmt:
				stateN++
				nextState := fmt.Sprintf("State%d", stateN)
				g.w("%s = maps:put('%s', ", nextState, s.Field)
				g.genExpr(s.Val)
				g.w(", %s),\n", g.currentState)
				g.currentState = nextState
			case *parser.ReturnStmt:
				g.w("From ! {result, ")
				if s.Val != nil {
					g.genExpr(s.Val)
				} else {
					g.w("ok")
				}
				g.w("},\n")
			default:
				g.genStmt(stmt, ",")
			}
		}

		hasReturn := false
		for _, stmt := range msg.Body.Stmts {
			if _, ok := stmt.(*parser.ReturnStmt); ok {
				hasReturn = true
			}
		}
		if !hasReturn {
			g.w("%sFrom ! {result, ok},\n", g.ind())
		}
		g.w("%s%s_loop(%s)\n", g.ind(), actorName, g.currentState)

		g.indent--

		if i < len(actor.Msgs)-1 {
			g.w("%s;\n", g.ind())
		}
	}

	g.w("%send.\n\n", g.ind())

	g.actorName = oldActor
}

func (g *Generator) genStruct(st *parser.StructDecl) {
	constructor := fmt.Sprintf("new_%s", strings.ToLower(st.Name))

	g.w("%s(", constructor)
	for i, field := range st.Fields {
		if i > 0 {
			g.w(", ")
		}
		g.w("%s", erlVar(field))
	}
	g.w(") ->\n")
	g.indent++
	g.w("%s#{", g.ind())
	for i, field := range st.Fields {
		if i > 0 {
			g.w(", ")
		}
		g.w("'%s' => %s", field, erlVar(field))
	}
	g.w("}%s\n", ".")
	g.indent--
	g.w("\n")
}

func (g *Generator) genExpr(expr parser.Expr) {
	switch e := expr.(type) {
	case *parser.IntLit:
		g.w("%s", e.Value)
	case *parser.FloatLit:
		g.w("%s", e.Value)
	case *parser.StrLit:
		g.w("\"%s\"", e.Value)
	case *parser.BoolLit:
		if e.Value {
			g.w("true")
		} else {
			g.w("false")
		}
	case *parser.Ident:
		if e.Name == "state" && g.actorName != "" {
			g.w("%s", g.currentStateVar())
		} else if erlName, ok := g.mutVars[e.Name]; ok {
			g.w("%s", erlName)
		} else {
			g.w("%s", erlVar(e.Name))
		}
	case *parser.BinExpr:
		g.w("(")
		g.genExpr(e.Left)
		g.w(" %s ", binOp(e.Op))
		g.genExpr(e.Right)
		g.w(")")
	case *parser.UnaryExpr:
		if e.Op == "not" {
			g.w("not (")
			g.genExpr(e.Expr)
			g.w(")")
		} else {
			g.w("(-")
			g.genExpr(e.Expr)
			g.w(")")
		}
	case *parser.CallExpr:
		g.genCall(e)
	case *parser.DotExpr:
		g.genDotExpr(e)
	case *parser.IndexExpr:
		if strLit, isStr := e.Index.(*parser.StrLit); isStr {
			g.w("maps:get('%s', ", strLit.Value)
			g.genExpr(e.Object)
			g.w(")")
		} else {
			g.w("lists:nth((")
			g.genExpr(e.Index)
			g.w(") + 1, ")
			g.genExpr(e.Object)
			g.w(")")
		}
	case *parser.ListLit:
		g.w("[")
		for i, elem := range e.Elems {
			if i > 0 {
				g.w(", ")
			}
			g.genExpr(elem)
		}
		g.w("]")
	case *parser.MapLit:
		g.w("#{")
		for i, key := range e.Keys {
			if i > 0 {
				g.w(", ")
			}
			g.w("%s => ", erlAtom(key))
			g.genExpr(e.Values[i])
		}
		g.w("}")
	case *parser.FuncLit:
		g.w("fun(")
		for i, param := range e.Params {
			if i > 0 {
				g.w(", ")
			}
			g.w("%s", erlVar(param))
		}
		g.w(") ->\n")
		g.indent++
		g.genBlock(e.Body, "")
		g.indent--
		g.w("%send", g.ind())
	}
}

func (g *Generator) genCall(call *parser.CallExpr) {
	if dot, ok := call.Func.(*parser.DotExpr); ok {
		if ident, ok := dot.Object.(*parser.Ident); ok {
			modName := ident.Name
			fnName := dot.Field


			if fnName == "spawn" {
				if g.actorNames[modName] {
					g.w("%s_start()", strings.ToLower(modName))
					return
				}
			}

			if g.structs[modName] && fnName == "new" {
				g.w("new_%s(", strings.ToLower(modName))
				for i, arg := range call.Args {
					if i > 0 {
						g.w(", ")
					}
					g.genExpr(arg)
				}
				g.w(")")
				return
			}

			if _, ok := g.actorVars[modName]; ok {
				g.receiveCount++
				resultVar := fmt.Sprintf("Rcv%d", g.receiveCount)
				g.w("begin ")
				g.w("%s ! {", erlVar(modName))
				g.w("%s", fnName)
				for _, arg := range call.Args {
					g.w(", ")
					g.genExpr(arg)
				}
				g.w(", self()}, ")
				g.w("receive {result, %s} -> %s end end", resultVar, resultVar)
				return
			}

			// stdlib io
			if modName == "io" && fnName == "println" {
				g.w("begin graft_print_item(")
				g.genExpr(call.Args[0])
				g.w("), io:format(\"~n\") end")
				return
			}
			if modName == "io" && fnName == "print" {
				g.w("graft_print_item(")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}
			if modName == "io" && fnName == "getLine" {
				g.w("(case io:get_line(")
				if len(call.Args) > 0 {
					g.genExpr(call.Args[0])
				} else {
					g.w("\"\"")
				}
				g.w(") of eof -> \"\"; Line -> string:trim(Line, both, \"\\n\") end)")
				return
			}

			// stdlib list
			if modName == "list" && fnName == "push" {
				g.w("lists:append(")
				g.genExpr(call.Args[0])
				g.w(", [")
				if len(call.Args) > 1 {
					g.genExpr(call.Args[1])
				}
				g.w("])")
				return
			}
			if modName == "list" && fnName == "remove" {
				g.w("lists:delete(")
				if len(call.Args) > 1 {
					g.genExpr(call.Args[1])
				}
				g.w(", ")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}
			if modName == "list" && fnName == "len" {
				g.w("length(")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}
			if modName == "list" && fnName == "last" {
				g.w("lists:nthtail(max(0, length(")
				g.genExpr(call.Args[0])
				g.w(") - ")
				g.genExpr(call.Args[1])
				g.w("), ")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}
			if modName == "list" && fnName == "join" {
				g.w("lists:flatten(lists:join(")
				g.genExpr(call.Args[1])
				g.w(", ")
				g.genExpr(call.Args[0])
				g.w("))")
				return
			}

			// stdlib string
			if modName == "string" && fnName == "len" {
				g.w("length(")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}
			if modName == "string" && fnName == "from_int" {
				g.w("integer_to_list(")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}

			// stdlib time
			if modName == "time" && fnName == "now" {
				g.w("integer_to_list(erlang:system_time(second))")
				return
			}

			// stdlib net (gen_tcp)
			if modName == "net" && fnName == "listen" {
				g.w("(case gen_tcp:listen(")
				g.genExpr(call.Args[0])
				g.w(", [binary, {active, false}, {packet, line}, {reuseaddr, true}]) of {ok, S} -> S; {error, E} -> error({net_listen, E}) end)")
				return
			}
			if modName == "net" && fnName == "accept" {
				g.w("(case gen_tcp:accept(")
				g.genExpr(call.Args[0])
				g.w(") of {ok, S} -> S; {error, E} -> error({net_accept, E}) end)")
				return
			}
			if modName == "net" && fnName == "send" {
				g.w("(case gen_tcp:send(")
				g.genExpr(call.Args[0])
				g.w(", iolist_to_binary([")
				g.genExpr(call.Args[1])
				g.w("])) of ok -> ok; {error, _} -> ok end)")
				return
			}
			if modName == "net" && fnName == "recv" {
				g.w("(case gen_tcp:recv(")
				g.genExpr(call.Args[0])
				g.w(", 0, 300000) of {ok, D} -> string:trim(binary_to_list(D)); {error, timeout} -> \"\"; {error, closed} -> \"\"; {error, E} -> error({net_recv, E}) end)")
				return
			}
			if modName == "net" && fnName == "close" {
				g.w("gen_tcp:close(")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}

			// stdlib process
			if modName == "process" && fnName == "spawn" {
				g.w("spawn(")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}
			if modName == "process" && fnName == "self" {
				g.w("self()")
				return
			}

			// stdlib map
			if modName == "map" && fnName == "new" {
				g.w("#{}")
				return
			}
			if modName == "map" && fnName == "put" {
				g.w("maps:put(")
				g.genExpr(call.Args[1])
				g.w(", ")
				g.genExpr(call.Args[2])
				g.w(", ")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}
			if modName == "map" && fnName == "get" {
				g.w("maps:get(")
				g.genExpr(call.Args[1])
				g.w(", ")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}
			if modName == "map" && fnName == "remove" {
				g.w("maps:remove(")
				g.genExpr(call.Args[1])
				g.w(", ")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}
			if modName == "map" && fnName == "to_list" {
				g.w("maps:to_list(")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}
			if modName == "map" && fnName == "from_list" {
				g.w("maps:from_list(")
				g.genExpr(call.Args[0])
				g.w(")")
				return
			}

			// Generic module:function
			g.w("%s:%s(", modName, fnName)
			for i, arg := range call.Args {
				if i > 0 {
					g.w(", ")
				}
				g.genExpr(arg)
			}
			g.w(")")
			return
		}
	}

	if ident, ok := call.Func.(*parser.Ident); ok {
		g.w("%s(", strings.ToLower(ident.Name))
	} else {
		g.genExpr(call.Func)
		g.w("(")
	}
	for i, arg := range call.Args {
		if i > 0 {
			g.w(", ")
		}
		g.genExpr(arg)
	}
	g.w(")")
}

func (g *Generator) genDotExpr(dot *parser.DotExpr) {
	if ident, ok := dot.Object.(*parser.Ident); ok {
		if ident.Name == "state" && g.actorName != "" {
			g.w("maps:get('%s', %s)", dot.Field, g.currentStateVar())
			return
		}
		erlName := erlVar(ident.Name)
		g.w("maps:get('%s', %s)", dot.Field, erlName)
		return
	}
	g.w("maps:get('%s', ", dot.Field)
	g.genExpr(dot.Object)
	g.w(")")
}

func (g *Generator) currentStateVar() string {
	if g.currentState != "" {
		return g.currentState
	}
	return "State"
}

func erlVar(s string) string {
	if len(s) == 0 {
		return s
	}
	return strings.ToUpper(s[:1]) + s[1:]
}

func erlAtom(s string) string {
	return fmt.Sprintf("'%s'", s)
}

func binOp(op string) string {
	switch op {
	case "and":
		return "andalso"
	case "or":
		return "orelse"
	case "mod":
		return "rem"
	case "!=":
		return "/="
	case "<=":
		return "=<"
	default:
		return op
	}
}
