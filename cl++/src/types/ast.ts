export type ASTNode = Program | FunctionDeclaration | Statement | Expression;

export type Statement =
  | ExpressionStatement
  | VariableDeclaration
  | IfStatement
  | WhileStatement
  | ReturnStatement
  | ImportDeclaration
  | ForInStatement

export type PrimaryExpression =
  StringLiteral | AtomLiteral | Identifier | BooleanLiteral | NumberLiteral | ArrayExpression | AnonymousFunctionExpression | ReceiveExpression;

export type Expression =
  | PrimaryExpression
  | MemberExpression
  | TupleExpression
  | CallExpression
  | BinaryExpression
  | ArrayExpression
  | TryExpression
  | LogicalExpression

export type SymbokKind = "variable" | "parameter" | "function" | "builtin" | "module";

export type ImportDeclaration = {
  type: "ImportDeclaration";
  source: StringLiteral;
};

export interface Program {
  type: "Program";
  body: (FunctionDeclaration | ImportDeclaration)[];
}

export interface ForInStatement {
  type: "ForInStatement",
  iterator: Identifier,
  iterable: Expression,
  body: Statement[]
}

export interface IfStatement {
  type: "IfStatement";
  condition: Expression;
  consequent: Statement[];
  alternate?: Statement[] | IfStatement;
}

export interface TupleExpression {
  type: "TupleExpression",
  elements: Expression[]
}

export interface LogicalExpression {
  type: "LogicalExpression"
  operator: "and" | "or"
  left: Expression,
  right: Expression
}

export interface TryExpression {
  type: "TryExpression";
  argument: Expression;
}

export interface ReceiveCase {
  type: "ReceiveCase",
  pattern: Expression,
  body: Statement[]
}

export interface ReceiveExpression {
  type: "ReceiveExpression"
  cases: ReceiveCase[]
}

export interface WhileStatement {
  type: "WhileStatement";
  condition: Expression;
  body: Statement[];
}

export interface ReturnStatement {
  type: "ReturnStatement";
  argument: Expression;
}

export interface ArrayExpression {
  type: "ArrayExpression";
  elements: Expression[];
}

export interface BinaryExpression {
  type: "BinaryExpression";
  left: Expression;
  operator: "==" | "~=" | "<" | ">" | "<=" | ">=" | "+" | "-" | "++";
  right: Expression;
}

export interface AtomLiteral {
  type: "AtomLiteral",
  value: string
}

export interface AnonymousFunctionExpression {
  type: "AnonymousFunctionExpression",
  params: Identifier[],
  body: Statement[]
}

export interface FunctionDeclaration {
  type: "FunctionDeclaration";
  name: Identifier;
  params: Identifier[];
  body: Statement[];
  isPublic: boolean;
}

export interface VariableDeclaration {
  type: "VariableDeclaration";
  name: Identifier;
  value: Expression;
}

export interface ExpressionStatement {
  type: "ExpressionStatement";
  expression: Expression;
}

export interface CallExpression {
  type: "CallExpression";
  callee: MemberExpression | PrimaryExpression;
  arguments: Expression[];
}

export interface MemberExpression {
  type: "MemberExpression";
  object: PrimaryExpression;
  property: Identifier;
  computed: boolean;
}

export interface StringLiteral {
  type: "StringLiteral";
  value: string;
}

export interface BooleanLiteral {
  type: "BooleanLiteral";
  value: boolean;
}

export interface NumberLiteral {
  type: "NumberLiteral";
  value: number;
}

export interface Identifier {
  type: "Identifier";
  name: string;
  symbolKind?: SymbokKind;
}
