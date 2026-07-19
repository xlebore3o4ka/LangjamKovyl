package parser

import (
	"fmt"
	"graft/lexer"
	"strings"
)

type Parser struct {
	tokens []lexer.Token
	pos    int
}

func New(tokens []lexer.Token) *Parser {
	return &Parser{tokens: tokens, pos: 0}
}

func (p *Parser) peek() lexer.Token {
	if p.pos >= len(p.tokens) {
		return lexer.Token{Type: lexer.EOF}
	}
	return p.tokens[p.pos]
}

func (p *Parser) advance() lexer.Token {
	tok := p.tokens[p.pos]
	p.pos++
	return tok
}

func (p *Parser) expect(tt lexer.TokenType) (lexer.Token, error) {
	tok := p.peek()
	if tok.Type != tt {
		return tok, fmt.Errorf("expected %v, got %v at line %d", tt, tok.Type, tok.Line)
	}
	return p.advance(), nil
}

func (p *Parser) match(tt lexer.TokenType) bool {
	if p.peek().Type == tt {
		p.advance()
		return true
	}
	return false
}

func (p *Parser) skipNewlines() {
	for p.peek().Type == lexer.NEWLINE {
		p.advance()
	}
}

func (p *Parser) Parse() (*Program, error) {
	prog := &Program{}

	for p.peek().Type != lexer.EOF {
		if p.peek().Type == lexer.KW_USE {
			stmt, err := p.parseUse()
			if err != nil {
				return nil, err
			}
			prog.Uses = append(prog.Uses, stmt)
		} else {
			top, err := p.parseTopLevel()
			if err != nil {
				return nil, err
			}
			prog.Tops = append(prog.Tops, top)
		}
	}

	return prog, nil
}

func (p *Parser) parseUse() (UseStmt, error) {
	tok := p.advance() // use
	p.skipNewlines()

	modParts := []string{}
	name, err := p.expect(lexer.IDENT)
	if err != nil {
		return UseStmt{}, err
	}
	modParts = append(modParts, name.Literal)

	for p.peek().Type == lexer.DOT {
		p.advance()
		part, err := p.expect(lexer.IDENT)
		if err != nil {
			return UseStmt{}, err
		}
		modParts = append(modParts, part.Literal)
	}

	return UseStmt{
		Module: strings.Join(modParts, "."),
		Line:   tok.Line,
	}, nil
}

func (p *Parser) parseTopLevel() (TopLevel, error) {
	tok := p.peek()
	switch tok.Type {
	case lexer.KW_FN:
		return p.parseFnDecl()
	case lexer.KW_ACTOR:
		return p.parseActorDecl()
	case lexer.KW_STRUCT:
		return p.parseStructDecl()
	default:
		return nil, fmt.Errorf("unexpected token %v at line %d", tok.Type, tok.Line)
	}
}

func (p *Parser) parseFnDecl() (*FnDecl, error) {
	tok := p.advance() // fn
	name, err := p.expect(lexer.IDENT)
	if err != nil {
		return nil, err
	}

	if _, err := p.expect(lexer.LPAREN); err != nil {
		return nil, err
	}

	params := []string{}
	if p.peek().Type != lexer.RPAREN {
	 paramName, err := p.expect(lexer.IDENT)
		if err != nil {
			return nil, err
		}
		params = append(params, paramName.Literal)
		for p.peek().Type == lexer.COMMA {
			p.advance()
			paramName, err = p.expect(lexer.IDENT)
			if err != nil {
				return nil, err
			}
			params = append(params, paramName.Literal)
		}
	}

	if _, err := p.expect(lexer.RPAREN); err != nil {
		return nil, err
	}

	body, err := p.parseBlock()
	if err != nil {
		return nil, err
	}

	return &FnDecl{
		Name:   name.Literal,
		Params: params,
		Body:   body,
		Line:   tok.Line,
	}, nil
}

func (p *Parser) parseActorDecl() (*ActorDecl, error) {
	tok := p.advance() // actor
	name, err := p.expect(lexer.IDENT)
	if err != nil {
		return nil, err
	}
	if _, err := p.expect(lexer.LBRACE); err != nil {
		return nil, err
	}

	msgs := []MsgDecl{}
	for p.peek().Type == lexer.KW_MSG {
		msg, err := p.parseMsgDecl()
		if err != nil {
			return nil, err
		}
		msgs = append(msgs, *msg)
	}

	if _, err := p.expect(lexer.RBRACE); err != nil {
		return nil, err
	}

	return &ActorDecl{
		Name: name.Literal,
		Msgs: msgs,
		Line: tok.Line,
	}, nil
}

func (p *Parser) parseMsgDecl() (*MsgDecl, error) {
	tok := p.advance() // msg
	name, err := p.expect(lexer.IDENT)
	if err != nil {
		return nil, err
	}

	if _, err := p.expect(lexer.LPAREN); err != nil {
		return nil, err
	}

	params := []string{}
	if p.peek().Type != lexer.RPAREN {
		paramName, err := p.expect(lexer.IDENT)
		if err != nil {
			return nil, err
		}
		params = append(params, paramName.Literal)
		for p.peek().Type == lexer.COMMA {
			p.advance()
			paramName, err = p.expect(lexer.IDENT)
			if err != nil {
				return nil, err
			}
			params = append(params, paramName.Literal)
		}
	}

	if _, err := p.expect(lexer.RPAREN); err != nil {
		return nil, err
	}

	body, err := p.parseBlock()
	if err != nil {
		return nil, err
	}

	return &MsgDecl{
		Name:   name.Literal,
		Params: params,
		Body:   body,
		Line:   tok.Line,
	}, nil
}

func (p *Parser) parseStructDecl() (*StructDecl, error) {
	tok := p.advance() // struct
	name, err := p.expect(lexer.IDENT)
	if err != nil {
		return nil, err
	}
	if _, err := p.expect(lexer.LBRACE); err != nil {
		return nil, err
	}

	fields := []string{}
	if p.peek().Type != lexer.RBRACE {
		field, err := p.expect(lexer.IDENT)
		if err != nil {
			return nil, err
		}
		fields = append(fields, field.Literal)
		for p.peek().Type == lexer.COMMA {
			p.advance()
			field, err = p.expect(lexer.IDENT)
			if err != nil {
				return nil, err
			}
			fields = append(fields, field.Literal)
		}
	}

	if _, err := p.expect(lexer.RBRACE); err != nil {
		return nil, err
	}

	return &StructDecl{
		Name:   name.Literal,
		Fields: fields,
		Line:   tok.Line,
	}, nil
}

func (p *Parser) parseBlock() (*Block, error) {
	if _, err := p.expect(lexer.LBRACE); err != nil {
		return nil, err
	}

	block := &Block{}
	for p.peek().Type != lexer.RBRACE && p.peek().Type != lexer.EOF {
		stmt, err := p.parseStatement()
		if err != nil {
			return nil, err
		}
		block.Stmts = append(block.Stmts, stmt)
	}

	if _, err := p.expect(lexer.RBRACE); err != nil {
		return nil, err
	}

	return block, nil
}

func (p *Parser) parseStatement() (Statement, error) {
	tok := p.peek()
	switch tok.Type {
	case lexer.KW_LET:
		return p.parseLet()
	case lexer.KW_MUT:
		return p.parseMut()
	case lexer.KW_RETURN:
		return p.parseReturn()
	case lexer.KW_IF:
		return p.parseIf()
	case lexer.KW_WHILE:
		return p.parseWhile()
	case lexer.KW_FOR:
		return p.parseFor()
	case lexer.KW_MATCH:
		return p.parseMatch()
	case lexer.KW_BREAK:
		p.advance()
		p.match(lexer.SEMICOLON)
		return &BreakStmt{Line: tok.Line}, nil
	case lexer.KW_CONTINUE:
		p.advance()
		p.match(lexer.SEMICOLON)
		return &ContinueStmt{Line: tok.Line}, nil
	default:
		return p.parseExprStmt()
	}
}

func (p *Parser) parseLet() (*LetStmt, error) {
	tok := p.advance() // let
	name, err := p.expect(lexer.IDENT)
	if err != nil {
		return nil, err
	}
	if _, err := p.expect(lexer.ASSIGN); err != nil {
		return nil, err
	}
	init, err := p.parseExpr()
	if err != nil {
		return nil, err
	}
	p.match(lexer.SEMICOLON)

	return &LetStmt{Name: name.Literal, Init: init, Line: tok.Line}, nil
}

func (p *Parser) parseMut() (*MutAssign, error) {
	tok := p.advance() // mut
	name, err := p.expect(lexer.IDENT)
	if err != nil {
		return nil, err
	}
	if _, err := p.expect(lexer.ASSIGN); err != nil {
		return nil, err
	}
	init, err := p.parseExpr()
	if err != nil {
		return nil, err
	}
	p.match(lexer.SEMICOLON)

	return &MutAssign{Name: name.Literal, Init: init, Line: tok.Line}, nil
}

func (p *Parser) parseReturn() (*ReturnStmt, error) {
	tok := p.advance() // return
	var val Expr
	if p.peek().Type != lexer.RBRACE && p.peek().Type != lexer.EOF {
		var err error
		val, err = p.parseExpr()
		if err != nil {
			return nil, err
		}
	}
	p.match(lexer.SEMICOLON)

	return &ReturnStmt{Val: val, Line: tok.Line}, nil
}

func (p *Parser) parseIf() (*IfStmt, error) {
	tok := p.advance() // if
	cond, err := p.parseExpr()
	if err != nil {
		return nil, err
	}
	then, err := p.parseBlock()
	if err != nil {
		return nil, err
	}

	var elseIf *IfStmt
	var els *Block

	if p.peek().Type == lexer.KW_ELSE {
		p.advance()
		if p.peek().Type == lexer.KW_IF {
			elseIf, err = p.parseIf()
			if err != nil {
				return nil, err
			}
		} else {
			els, err = p.parseBlock()
			if err != nil {
				return nil, err
			}
		}
	}

	return &IfStmt{
		Cond:   cond,
		Then:   then,
		ElseIf: elseIf,
		Else:   els,
		Line:   tok.Line,
	}, nil
}

func (p *Parser) parseWhile() (*WhileStmt, error) {
	tok := p.advance() // while
	cond, err := p.parseExpr()
	if err != nil {
		return nil, err
	}
	body, err := p.parseBlock()
	if err != nil {
		return nil, err
	}

	return &WhileStmt{Cond: cond, Body: body, Line: tok.Line}, nil
}

func (p *Parser) parseFor() (*ForStmt, error) {
	tok := p.advance() // for
	varName, err := p.expect(lexer.IDENT)
	if err != nil {
		return nil, err
	}
	if _, err := p.expect(lexer.KW_IN); err != nil {
		return nil, err
	}
	seq, err := p.parseExpr()
	if err != nil {
		return nil, err
	}
	body, err := p.parseBlock()
	if err != nil {
		return nil, err
	}

	return &ForStmt{Var: varName.Literal, Seq: seq, Body: body, Line: tok.Line}, nil
}

func (p *Parser) parseMatch() (*MatchStmt, error) {
	tok := p.advance() // match
	target, err := p.parseExpr()
	if err != nil {
		return nil, err
	}
	if _, err := p.expect(lexer.LBRACE); err != nil {
		return nil, err
	}

	var cases []MatchCase
	for p.peek().Type != lexer.RBRACE {
		if _, err := p.expect(lexer.KW_CASE); err != nil {
			return nil, err
		}

		var pattern Expr
		if p.peek().Type == lexer.KW_WILDCARD {
			p.advance()
			pattern = nil
		} else {
			pattern, err = p.parseExpr()
			if err != nil {
				return nil, err
			}
		}

		body, err := p.parseBlock()
		if err != nil {
			return nil, err
		}

		cases = append(cases, MatchCase{Pattern: pattern, Body: body})
	}

	if _, err := p.expect(lexer.RBRACE); err != nil {
		return nil, err
	}

	return &MatchStmt{Target: target, Cases: cases, Line: tok.Line}, nil
}

func (p *Parser) parseExprStmt() (Statement, error) {
	expr, err := p.parseExpr()
	if err != nil {
		return nil, err
	}
	line := p.peek().Line

	if p.peek().Type == lexer.ASSIGN {
		p.advance()
		val, err := p.parseExpr()
		if err != nil {
			return nil, err
		}
		p.match(lexer.SEMICOLON)

		// Simple variable assignment
		if ident, ok := expr.(*Ident); ok {
			return &AssignStmt{Name: ident.Name, Val: val, Line: line}, nil
		}
		// Dot assignment (e.g. state.users = ...)
		if dot, ok := expr.(*DotExpr); ok {
			return &DotAssignStmt{Object: dot.Object, Field: dot.Field, Val: val, Line: line}, nil
		}
		return nil, fmt.Errorf("invalid assignment target at line %d", line)
	}

	p.match(lexer.SEMICOLON)
	return &ExprStmt{Expr: expr, Line: line}, nil
}

func (p *Parser) parseExpr() (Expr, error) {
	return p.parseOr()
}

func (p *Parser) parseOr() (Expr, error) {
	left, err := p.parseAnd()
	if err != nil {
		return nil, err
	}

	for p.peek().Type == lexer.KW_OR {
		p.advance()
		right, err := p.parseAnd()
		if err != nil {
			return nil, err
		}
		left = &BinExpr{Op: "or", Left: left, Right: right}
	}

	return left, nil
}

func (p *Parser) parseAnd() (Expr, error) {
	left, err := p.parseNot()
	if err != nil {
		return nil, err
	}

	for p.peek().Type == lexer.KW_AND {
		p.advance()
		right, err := p.parseNot()
		if err != nil {
			return nil, err
		}
		left = &BinExpr{Op: "and", Left: left, Right: right}
	}

	return left, nil
}

func (p *Parser) parseNot() (Expr, error) {
	if p.peek().Type == lexer.KW_NOT {
		p.advance()
		expr, err := p.parseNot()
		if err != nil {
			return nil, err
		}
		return &UnaryExpr{Op: "not", Expr: expr}, nil
	}
	return p.parseComparison()
}

func (p *Parser) parseComparison() (Expr, error) {
	left, err := p.parseAddition()
	if err != nil {
		return nil, err
	}

	for {
		tok := p.peek()
		switch tok.Type {
		case lexer.EQ_EQ, lexer.NOT_EQ, lexer.LT, lexer.GT, lexer.LT_EQ, lexer.GT_EQ:
			p.advance()
			right, err := p.parseAddition()
			if err != nil {
				return nil, err
			}
			left = &BinExpr{Op: tok.Literal, Left: left, Right: right}
		default:
			return left, nil
		}
	}
}

func (p *Parser) parseAddition() (Expr, error) {
	left, err := p.parseMultiplication()
	if err != nil {
		return nil, err
	}

	for {
		tok := p.peek()
		switch tok.Type {
		case lexer.PLUS, lexer.MINUS, lexer.PLUS_PLUS:
			p.advance()
			right, err := p.parseMultiplication()
			if err != nil {
				return nil, err
			}
			left = &BinExpr{Op: tok.Literal, Left: left, Right: right}
		default:
			return left, nil
		}
	}
}

func (p *Parser) parseMultiplication() (Expr, error) {
	left, err := p.parseUnary()
	if err != nil {
		return nil, err
	}

	for {
		tok := p.peek()
		switch tok.Type {
		case lexer.STAR, lexer.SLASH, lexer.KW_MOD:
			p.advance()
			right, err := p.parseUnary()
			if err != nil {
				return nil, err
			}
			left = &BinExpr{Op: tok.Literal, Left: left, Right: right}
		default:
			return left, nil
		}
	}
}

func (p *Parser) parseUnary() (Expr, error) {
	if p.peek().Type == lexer.MINUS {
		p.advance()
		expr, err := p.parseUnary()
		if err != nil {
			return nil, err
		}
		return &UnaryExpr{Op: "-", Expr: expr}, nil
	}
	return p.parsePostfixExpr()
}

func (p *Parser) parsePostfixExpr() (Expr, error) {
	expr, err := p.parsePrimary()
	if err != nil {
		return nil, err
	}
	return p.parsePostfix(expr)
}

func (p *Parser) parsePrimary() (Expr, error) {
	tok := p.peek()

	switch tok.Type {
	case lexer.INT_LIT:
		p.advance()
		return &IntLit{Value: tok.Literal}, nil

	case lexer.FLOAT_LIT:
		p.advance()
		return &FloatLit{Value: tok.Literal}, nil

	case lexer.STRING_LIT:
		p.advance()
		return &StrLit{Value: tok.Literal}, nil

	case lexer.KW_TRUE:
		p.advance()
		return &BoolLit{Value: true}, nil

	case lexer.KW_FALSE:
		p.advance()
		return &BoolLit{Value: false}, nil

	case lexer.IDENT:
		p.advance()
		return &Ident{Name: tok.Literal}, nil

	case lexer.LBRACKET:
		return p.parseListLit()

	case lexer.LBRACE:
		return p.parseMapLit()

	case lexer.KW_FN:
		return p.parseFuncLit()

	case lexer.LPAREN:
		p.advance()
		expr, err := p.parseExpr()
		if err != nil {
			return nil, err
		}
		if _, err := p.expect(lexer.RPAREN); err != nil {
			return nil, err
		}
		return expr, nil

	default:
		return nil, fmt.Errorf("unexpected token %v at line %d", tok.Type, tok.Line)
	}
}

func (p *Parser) parsePostfix(left Expr) (Expr, error) {
	for {
		switch p.peek().Type {
		case lexer.DOT:
			p.advance()
			field, err := p.expect(lexer.IDENT)
			if err != nil {
				return nil, err
			}
			left = &DotExpr{Object: left, Field: field.Literal}

		case lexer.LPAREN:
			p.advance()
			args := []Expr{}
			if p.peek().Type != lexer.RPAREN {
				arg, err := p.parseExpr()
				if err != nil {
					return nil, err
				}
				args = append(args, arg)
				for p.peek().Type == lexer.COMMA {
					p.advance()
					arg, err = p.parseExpr()
					if err != nil {
						return nil, err
					}
					args = append(args, arg)
				}
			}
			if _, err := p.expect(lexer.RPAREN); err != nil {
				return nil, err
			}
			left = &CallExpr{Func: left, Args: args}

		case lexer.LBRACKET:
			p.advance()
			idx, err := p.parseExpr()
			if err != nil {
				return nil, err
			}
			if _, err := p.expect(lexer.RBRACKET); err != nil {
				return nil, err
			}
			left = &IndexExpr{Object: left, Index: idx}

		default:
			return left, nil
		}
	}
}

func (p *Parser) parseListLit() (Expr, error) {
	p.advance() // [
	elems := []Expr{}
	if p.peek().Type != lexer.RBRACKET {
		elem, err := p.parseExpr()
		if err != nil {
			return nil, err
		}
		elems = append(elems, elem)
		for p.peek().Type == lexer.COMMA {
			p.advance()
			elem, err = p.parseExpr()
			if err != nil {
				return nil, err
			}
			elems = append(elems, elem)
		}
	}
	if _, err := p.expect(lexer.RBRACKET); err != nil {
		return nil, err
	}
	return &ListLit{Elems: elems}, nil
}

func (p *Parser) parseMapLit() (Expr, error) {
	p.advance() // {
	keys := []string{}
	values := []Expr{}

	if p.peek().Type != lexer.RBRACE {
		key, err := p.expect(lexer.STRING_LIT)
		if err != nil {
			return nil, err
		}
		keys = append(keys, key.Literal)
		if _, err := p.expect(lexer.COLON); err != nil {
			return nil, err
		}
		val, err := p.parseExpr()
		if err != nil {
			return nil, err
		}
		values = append(values, val)

		for p.peek().Type == lexer.COMMA {
			p.advance()
			key, err := p.expect(lexer.STRING_LIT)
			if err != nil {
				return nil, err
			}
			keys = append(keys, key.Literal)
			if _, err := p.expect(lexer.COLON); err != nil {
				return nil, err
			}
			val, err := p.parseExpr()
			if err != nil {
				return nil, err
			}
			values = append(values, val)
		}
	}

	if _, err := p.expect(lexer.RBRACE); err != nil {
		return nil, err
	}

	return &MapLit{Keys: keys, Values: values}, nil
}

func (p *Parser) parseFuncLit() (*FuncLit, error) {
	p.advance() // fn
	if _, err := p.expect(lexer.LPAREN); err != nil {
		return nil, err
	}
	params := []string{}
	if p.peek().Type != lexer.RPAREN {
		name, err := p.expect(lexer.IDENT)
		if err != nil {
			return nil, err
		}
		params = append(params, name.Literal)
		for p.peek().Type == lexer.COMMA {
			p.advance()
			name, err = p.expect(lexer.IDENT)
			if err != nil {
				return nil, err
			}
			params = append(params, name.Literal)
		}
	}
	if _, err := p.expect(lexer.RPAREN); err != nil {
		return nil, err
	}
	body, err := p.parseBlock()
	if err != nil {
		return nil, err
	}
	return &FuncLit{Params: params, Body: body}, nil
}
