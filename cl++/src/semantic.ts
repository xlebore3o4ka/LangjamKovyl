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
    this.globalScope.define("list_to_integer", "builtin")
    this.globalScope.define("integer_to_list", "builtin")
    this.globalScope.define("element", "builtin")
    this.globalScope.define("lists", "builtin")
    this.globalScope.define("binary_to_list", "builtin")
    this.globalScope.define("string", "builtin")
    this.globalScope.define("erlang", "builtin")
    this.globalScope.define("byte_size", "builtin")
    this.globalScope.define("binary_part", "builtin")
    this.globalScope.define("unicode", "builtin")
    this.globalScope.define("timer", "builtin")
    this.globalScope.define("random", "builtin")
    this.globalScope.define("print", "builtin")
    this.globalScope.define("spawn", "builtin")
    this.globalScope.define("calendar", "builtin")
    this.globalScope.define("register", "builtin")
    this.globalScope.define("list_to_atom", "builtin")
    this.globalScope.define("self", "builtin")
    this.globalScope.define("whereis", "builtin")
    this.globalScope.define("exit", "builtin")
    this.globalScope.define("length", "builtin")
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

      case "AnonymousFunctionExpression":
        const previousScopeAnonym = this.currentScope
        this.currentScope = new Scope(this.currentScope)

        for (const param of node.params) {
          this.currentScope.define(param.name, "parameter")
          param.symbolKind = "parameter"
        }

        for (const stmt of node.body) this.analyze(stmt)

        this.currentScope = previousScopeAnonym
        break

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

      case "ReceiveExpression":
        for (const receiveCase of node.cases) {
          const previousScopeReceive = this.currentScope
          this.currentScope = new Scope(this.currentScope)

          this.analyzePattern(receiveCase.pattern)

          for (const stmt of receiveCase.body) this.analyze(stmt)

          this.currentScope = previousScopeReceive
        }
        break

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

      case "TryExpression":
        this.analyze(node.argument);
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

      case "ForInStatement":
        this.analyze(node.iterable)

        const prevScope = this.currentScope
        this.currentScope = new Scope(prevScope)

        this.currentScope.define(node.iterator.name, "variable")

        for (const stmt of node.body) this.analyze(stmt)

        this.currentScope = prevScope
        break

      case "CallExpression":
        this.analyze(node.callee);
        for (const arg of node.arguments) this.analyze(arg);
        break;

      case "LogicalExpression":
        this.analyze(node.left)
        this.analyze(node.right)
        break

      case "TupleExpression":
        for (const element of node.elements) this.analyze(element);
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

  private analyzePattern(node: ASTNode): void {
    if (node.type === "Identifier") {
      if (!this.currentScope.resolve(node.name)) this.currentScope.define(node.name, "variable")
      node.symbolKind = "variable"
    } else if (node.type === "TupleExpression" || node.type === "ArrayExpression") {
      for (const element of node.elements) this.analyzePattern(element)
    } else this.analyze(node)
  }
}

export default SemanticAnalyzer;
