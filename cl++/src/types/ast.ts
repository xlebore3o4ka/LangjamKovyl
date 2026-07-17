export type ASTNode = Program | FunctionDeclaration | Statement | Expression;

export type Statement =
  | ExpressionStatement
  | VariableDeclaration
  | IfStatement
  | WhileStatement
  | ReturnStatement
  | ImportDeclaration;

export type PrimaryExpression =
  StringLiteral | AtomLiteral | Identifier | BooleanLiteral | NumberLiteral | ArrayExpression;

export type Expression =
  | PrimaryExpression
  | MemberExpression
  | CallExpression
  | BinaryExpression
  | ArrayExpression;

export type SymbokKind = "variable" | "parameter" | "function" | "builtin" | "module";

export type ImportDeclaration = {
  type: "ImportDeclaration";
  source: StringLiteral;
};

export interface Program {
  type: "Program";
  body: (FunctionDeclaration | ImportDeclaration)[];
}

export interface IfStatement {
  type: "IfStatement";
  condition: Expression;
  consequent: Statement[];
  alternate?: Statement[] | IfStatement;
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
