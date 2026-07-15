import { type ASTNode, type SymbokKind } from "./types/index.js";
import exitWithError from "./utils/exitWithError.js";

export interface SymbolInfo {
  name: string;
  kind: SymbokKind;
}

class Scope {
  private symbols = new Map<string, SymbolInfo>();
  private parent?: Scope | undefined;

  constructor(parent?: Scope) {
    this.parent = parent;
  }

  public define(name: string, kind: SymbokKind): void {
    if (this.symbols.has(name))
      exitWithError(`Identifier ${name} is already defined`);
    this.symbols.set(name, { name, kind });
  }

  public resolve(name: string): SymbolInfo | undefined {
    return this.symbols.get(name) || this.parent?.resolve(name);
  }
}

class SemanticAnalyzer {
  private globalScope = new Scope();
  private currentScope = this.globalScope;

  constructor() {
    this.globalScope.define("io", "builtin");
  }

  public analyze(node: ASTNode): void {
    switch (node.type) {
      case "Program":
        for (const func of node.body) {
          this.globalScope.define(func.name.name, "function");
          func.name.symbolKind = "function";
        }

        for (const func of node.body) this.analyze(func);
        break;

      case "FunctionDeclaration":
        const previousScope = this.currentScope;
        this.currentScope = new Scope(this.currentScope);

        for (const param of node.params) {
          this.currentScope.define(param.name, "parameter");
          param.symbolKind = "parameter";
        }

        for (const stmt of node.body) this.analyze(stmt);
        this.currentScope = previousScope;
        break;

      case "VariableDeclaration":
        this.analyze(node.value);
        this.currentScope.define(node.name.name, "variable");
        node.name.symbolKind = "variable";
        break;

      case "WhileStatement":
        this.analyze(node.condition);

        const previousScopeWhile = this.currentScope;
        this.currentScope = new Scope(this.currentScope);
        for (const stmt of node.body) this.analyze(stmt);
        this.currentScope = previousScopeWhile;
        break;

      case "ExpressionStatement":
        this.analyze(node.expression);
        break;

      case "BinaryExpression":
        this.analyze(node.left);
        this.analyze(node.right);
        break;

      case "IfStatement":
        this.analyze(node.condition);

        const previousScopeIf = this.currentScope;
        this.currentScope = new Scope(this.currentScope);
        for (const stmt of node.consequent) this.analyze(stmt);
        this.currentScope = previousScopeIf;

        if (!node.alternate) break;

        if (Array.isArray(node.alternate)) {
          const previousScopeElse = this.currentScope;
          this.currentScope = new Scope(this.currentScope);
          for (const stmt of node.alternate) this.analyze(stmt);
          this.currentScope = previousScopeElse;
        } else {
          this.analyze(node.alternate);
        }

        break;

      case "CallExpression":
        this.analyze(node.callee);
        for (const arg of node.arguments) this.analyze(arg);
        break;

      case "MemberExpression":
        this.analyze(node.object);
        break;

      case "Identifier":
        const symbol = this.currentScope.resolve(node.name);
        if (!symbol) exitWithError(`Identifier ${node.name} is not defined`);
        node.symbolKind = symbol.kind;
        break;

      case "BooleanLiteral":
      case "StringLiteral":
      case "NumberLiteral":
        break;
    }
  }
}

export { Scope, SemanticAnalyzer };
