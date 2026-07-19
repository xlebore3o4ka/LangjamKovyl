package parser

type Program struct {
	Uses    []UseStmt
	Tops    []TopLevel
}

type UseStmt struct {
	Module string
	Line   int
}

type TopLevel interface {
	topLevelNode()
	Pos() (int, int)
}

func (*FnDecl) topLevelNode()     {}
func (*ActorDecl) topLevelNode()  {}
func (*StructDecl) topLevelNode() {}

type FnDecl struct {
	Name   string
	Params []string
	Body   *Block
	Line   int
}

func (f *FnDecl) Pos() (int, int) { return f.Line, 0 }

type ActorDecl struct {
	Name string
	Msgs []MsgDecl
	Line int
}

func (a *ActorDecl) Pos() (int, int) { return a.Line, 0 }

type MsgDecl struct {
	Name   string
	Params []string
	Body   *Block
	Line   int
}

type StructDecl struct {
	Name   string
	Fields []string
	Line   int
}

func (s *StructDecl) Pos() (int, int) { return s.Line, 0 }

type Statement interface {
	stmtNode()
}

type Block struct {
	Stmts []Statement
}

func (*LetStmt) stmtNode()      {}
func (*MutAssign) stmtNode()    {}
func (*ReturnStmt) stmtNode()   {}
func (*IfStmt) stmtNode()       {}
func (*WhileStmt) stmtNode()    {}
func (*ForStmt) stmtNode()      {}
func (*ExprStmt) stmtNode()     {}
func (*AssignStmt) stmtNode()   {}
func (*DotAssignStmt) stmtNode() {}
func (*BreakStmt) stmtNode()    {}
func (*ContinueStmt) stmtNode() {}
func (*MatchStmt) stmtNode()    {}

type BreakStmt struct {
	Line int
}

type ContinueStmt struct {
	Line int
}

type LetStmt struct {
	Name string
	Init Expr
	Line int
}

type MutAssign struct {
	Name string
	Init Expr
	Line int
}

type AssignStmt struct {
	Name string
	Val  Expr
	Line int
}

type DotAssignStmt struct {
	Object Expr
	Field  string
	Val    Expr
	Line   int
}

type ReturnStmt struct {
	Val  Expr
	Line int
}

type IfStmt struct {
	Cond   Expr
	Then   *Block
	ElseIf *IfStmt // else if chain
	Else   *Block
	Line   int
}

type WhileStmt struct {
	Cond Expr
	Body *Block
	Line int
}

type ForStmt struct {
	Var  string
	Seq  Expr
	Body *Block
	Line int
}

type MatchCase struct {
	Pattern Expr // nil for wildcard _
	Body    *Block
}

type MatchStmt struct {
	Target Expr
	Cases  []MatchCase
	Line   int
}

type ExprStmt struct {
	Expr Expr
	Line int
}

type Expr interface {
	exprNode()
}

func (*BinExpr) exprNode()     {}
func (*UnaryExpr) exprNode()   {}
func (*IntLit) exprNode()      {}
func (*FloatLit) exprNode()    {}
func (*StrLit) exprNode()      {}
func (*BoolLit) exprNode()     {}
func (*Ident) exprNode()       {}
func (*CallExpr) exprNode()    {}
func (*DotExpr) exprNode()     {}
func (*IndexExpr) exprNode()   {}
func (*ListLit) exprNode()     {}
func (*MapLit) exprNode()      {}
func (*FuncLit) exprNode()     {}

type BinExpr struct {
	Op    string
	Left  Expr
	Right Expr
}

type UnaryExpr struct {
	Op   string
	Expr Expr
}

type IntLit struct {
	Value string
}

type FloatLit struct {
	Value string
}

type StrLit struct {
	Value string
}

type BoolLit struct {
	Value bool
}

type Ident struct {
	Name string
}

type CallExpr struct {
	Func Expr
	Args []Expr
}

type DotExpr struct {
	Object Expr
	Field  string
}

type IndexExpr struct {
	Object Expr
	Index  Expr
}

type ListLit struct {
	Elems []Expr
}

type MapLit struct {
	Keys   []string
	Values []Expr
}

type FuncLit struct {
	Params []string
	Body   *Block
}
