export type ASTNode = Program | FunctionDeclaration | Statement | Expression;

export type Statement = ExpressionStatement | VariableDeclaration | IfStatement;

export type PrimaryExpression = StringLiteral | Identifier;

export type Expression =
  PrimaryExpression | MemberExpression | CallExpression | BinaryExpression;

export type SymbokKind = "variable" | "parameter" | "function" | "builtin";

export interface Program {
  type: "Program";
  body: FunctionDeclaration[];
}

export interface IfStatement {
  type: "IfStatement";
  condition: Expression;
  consequent: Statement[];
  alternate?: Statement[] | IfStatement;
}

export interface BinaryExpression {
  type: "BinaryExpression";
  left: Expression;
  operator: "==" | "~=" | "<" | ">" | "<=" | ">=";
  right: Expression;
}

export interface FunctionDeclaration {
  type: "FunctionDeclaration";
  name: Identifier;
  params: Identifier[];
  body: Statement[];
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
}

export interface StringLiteral {
  type: "StringLiteral";
  value: string;
}

export interface Identifier {
  type: "Identifier";
  name: string;
  symbolKind?: SymbokKind;
}
