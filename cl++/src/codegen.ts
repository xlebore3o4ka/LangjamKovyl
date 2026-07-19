import stdlib from "./stdlib.js";
import { type ASTNode, type FunctionDeclaration } from "./types/index.js";

const capitalize = (str: string): string =>
  str.charAt(0).toUpperCase() + str.slice(1);

const generate = (node: ASTNode, moduleName: string = "main"): string => {
  switch (node.type) {
    case "Program":
      const functions = node.body.filter(
        (n): n is FunctionDeclaration => n.type === "FunctionDeclaration"
      )

      const exportedFunctions = functions.filter(f => f.isPublic)

      const exports = exportedFunctions.map(f => `${f.name.name}/${f.params.length}`).join(", ")

      const exportHeader = exports.length > 0 ? `-export([${exports}]).\n\n` : "\n";
      const moduleHeader = `-module(${moduleName}).\n${exportHeader}`

      const body = node.body.map(e => generate(e, moduleName)).join("\n\n");


      return moduleHeader + body

    case "FunctionDeclaration":
      const funcName = node.name.name;
      const stmts = node.body.map((e) => generate(e, moduleName)).join(",\n    ");
      const params = node.params.map((e) => generate(e, moduleName)).join(",");

      const funcBody = stmts.length > 0 ? stmts : "ok";

      return `${funcName}(${params}) ->\n    try\n        ${funcBody}\n    catch\n        throw:{'__clx_return', ReturnValue} -> \n        ReturnValue\n    end.`;

    case "TupleExpression":
    return `{${node.elements.map((e) => generate(e, moduleName)).join(", ")}}`

    case "AtomLiteral":
      return `${node.value}`

    case "ForInStatement":
      const iterName = capitalize(node.iterator.name)
      const iterableVal = generate(node.iterable, moduleName)
      const forBody = node.body.map(stmt => generate(stmt, moduleName)).join(",\n    ")

      return `lists:foreach(fun(${iterName}) ->\n    ${forBody}\nend, ${iterableVal})`

    case "ReceiveExpression":
      const casesCode = node.cases.map(c => {
        const pattern = generate(c.pattern, moduleName)
        const bodyStmts = c.body.map(e => generate(e, moduleName)).join(",\n        ")
        const bodyCode = bodyStmts.length > 0 ? bodyStmts : "ok"
        return `${pattern} ->\n        ${bodyCode}`
      }).join(";\n    ")

      return `receive\n ${casesCode}\nend`

    case "TryExpression":
      const innerExpression = generate(node.argument, moduleName);

      return `try ${innerExpression} of __TryRes -> {ok, __TryRes} catch _:__TryErr -> {error, __TryErr} end`

    case "ReturnStatement":
      const arg = generate(node.argument);
      return `throw({'__clx_return', ${arg}})`;

    case "ImportDeclaration":
      return "";

    case "LogicalExpression":
      const leftLogic = generate(node.left, moduleName)
      const rightLogic = generate(node.right, moduleName)
      const logicOperator = node.operator === "and" ? "andalso" : "orelse"

      return `(${leftLogic} ${logicOperator} ${rightLogic})`

    case "BinaryExpression":
      const left = generate(node.left);
      const right = generate(node.right);
      let operator: string = node.operator;

      if (operator === "~=") {
        operator = "/=";
      } else if (operator === "<=") {
        operator = "=<"
      }

      return `${left} ${operator} ${right}`;

    case "ArrayExpression":
      const elements = node.elements.map((e) => generate(e, moduleName)).join(", ");
      return `[${elements}]`;

    case "WhileStatement":
      const cond = generate(node.condition);
      const loopBody = node.body.map((e) => generate(e, moduleName)).join(",\n    ");
      const loopBodyCode = loopBody.length > 0 ? loopBody : "ok";
      return `(fun Loop() ->
        case clx_std:to_boolean(${cond}) of
            true ->
                ${loopBodyCode},
                Loop();
            _ ->
                ok
        end
    end)()`;

    case "BooleanLiteral":
      return node.value ? "true" : "false";

    case "NumberLiteral":
      return node.value.toString();

    case "IfStatement":
      const condition = generate(node.condition);
      const consStr = node.consequent.map((e) => generate(e, moduleName)).join(",\n    ");
      const consequent = consStr.length > 0 ? consStr : "ok";

      let alternate = "ok";
      if (node.alternate) {
        if (Array.isArray(node.alternate)) {
          const altStr = node.alternate.map((e) => generate(e, moduleName)).join(",\n    ");
          alternate = altStr.length > 0 ? altStr : "ok";
        } else {
          alternate = generate(node.alternate, moduleName);
        }
      }

      return `case clx_std:to_boolean(${condition}) of\n    true -> \n        ${consequent};\n    _ ->\n        ${alternate}\nend`;

    case "VariableDeclaration":
      const varName = capitalize(node.name.name);
      return `${varName} = ${generate(node.value, moduleName)}`;

    case "CallExpression":
      const callee = generate(node.callee, moduleName);
      const args = node.arguments.map((e) => generate(e, moduleName)).join(", ");

      if (stdlib[callee]) return stdlib[callee](args);

      return `${callee}(${args})`;

    case "AnonymousFunctionExpression":
      const anonymParams = node.params.map((e) => generate(e, moduleName)).join(", ")
      const anonymStmts = node.body.map(e => generate(e, moduleName)).join(",\n        ")


      const anonymBody = anonymStmts.length > 0 ? anonymStmts : "ok"

      return `fun(${anonymParams}) ->\n    try\n        ${anonymBody}\n    catch\n        throw:{'__clx_return', AnonymReturnValue} -> \n        AnonymReturnValue\n        end\n    end`

    case "MemberExpression":
      if (node.computed) {
        const object = generate(node.object, moduleName);
        const index = generate(node.property, moduleName);
        return `clx_std:get_element(${object}, ${index})`;
      } else {
        const object = generate(node.object, moduleName);
        const prop = generate(node.property, moduleName);
        return `${object}:${prop}`;
      }

    case "StringLiteral":
      return `"${node.value}"`;

    case "Identifier":
      if (node.symbolKind === "variable" || node.symbolKind === "parameter")
        return capitalize(node.name);

      return `${node.name}`;

    case "ExpressionStatement":
      return generate(node.expression, moduleName);
  }
};

export default generate;
