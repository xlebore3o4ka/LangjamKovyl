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
    this.globalScope.define("gen_tcp", "builtin")
    this.globalScope.define("list_to_binary", "builtin")
    this.globalScope.define("element", "builtin")

  }

  public analyze(node: ASTNode): void {
    switch (node.type) {
      case "Program":
        for (const decl of node.body) {
          if (decl.type === "FunctionDeclaration") {
            this.globalScope.define(decl.name.name, "function");
            decl.name.symbolKind = "function";
          } else if (decl.type === "ImportDeclaration") {
            this.globalScope.define(decl.source.value, "module");
          }
        }

        for (const decl of node.body) if (decl.type === "FunctionDeclaration") this.analyze(decl);

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

      case "ReturnStatement":
        this.analyze(node.argument);
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

      case "ArrayExpression":
        for (const element of node.elements) this.analyze(element);
        break;

      case "MemberExpression":
        this.analyze(node.object);
        break;

      case "Identifier":
        const symbol = this.currentScope.resolve(node.name);
        if (!symbol) exitWithError(`Identifier ${node.name} is not defined`);
        node.symbolKind = symbol.kind;
        break;

      case "ImportDeclaration":
      case "AtomLiteral":
      case "BooleanLiteral":
      case "StringLiteral":
      case "NumberLiteral":
        break;
    }
  }
}

export default SemanticAnalyzer;
