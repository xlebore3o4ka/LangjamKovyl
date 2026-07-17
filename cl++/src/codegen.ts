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

    case "AtomLiteral":
      return `${node.value}`

    case "ReturnStatement":
      const arg = generate(node.argument);
      return `throw({'__clx_return', ${arg}})`;

    case "ImportDeclaration":
      return "";

    case "BinaryExpression":
      const left = generate(node.left);
      const right = generate(node.right);
      const operator = node.operator === "~=" ? "/=" : node.operator;

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

      let alternate = "";
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

    case "MemberExpression":
      if (node.computed) {
        const object = generate(node.object, moduleName);
        const index = generate(node.property, moduleName);
        return `lists:nth(${index}, ${object})`;
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
