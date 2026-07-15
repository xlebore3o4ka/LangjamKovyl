import stdlib from "./stdlib.js";
import { type ASTNode } from "./types/index.js";

const capitalize = (str: string): string =>
  str.charAt(0).toUpperCase() + str.slice(1);

const generate = (node: ASTNode): string => {
  switch (node.type) {
    case "Program":
      const moduleHeader = `-module(main).\n-export([start/0]).\n\n`;
      const body = node.body.map(generate).join("\n\n");
      return moduleHeader + body;

    case "FunctionDeclaration":
      const funcName = node.name.name;
      const stmts = node.body.map(generate).join(",\n    ");
      return `${funcName}() ->\n    ${stmts}.`;

    case "BinaryExpression":
      const left = generate(node.left);
      const right = generate(node.right);
      const operator = node.operator === "~=" ? "/=" : node.operator;

      return `${left} ${operator} ${right}`;

    case "WhileStatement":
      const cond = generate(node.condition);
      const loopBody = node.body.map(generate).join(",\n    ");
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
      const consStr = node.consequent.map(generate).join(",\n    ");
      const consequent = consStr.length > 0 ? consStr : "ok";

      let alternate = "";
      if (node.alternate) {
        if (Array.isArray(node.alternate)) {
          const altStr = node.alternate.map(generate).join(",\n    ");
          alternate = altStr.length > 0 ? altStr : "ok";
        } else {
          alternate = generate(node.alternate);
        }
      }

      return `case clx_std:to_boolean(${condition}) of\n    true -> \n        ${consequent};\n    _ ->\n        ${alternate}\nend`;

    case "VariableDeclaration":
      const varName = capitalize(node.name.name);
      return `${varName} = ${generate(node.value)}`;

    case "CallExpression":
      const callee = generate(node.callee);
      const args = node.arguments.map(generate).join(", ");

      if (stdlib[callee]) return stdlib[callee](args);

      return `${callee}(${args})`;

    case "MemberExpression":
      const objName =
        node.object.type === "Identifier"
          ? node.object.name
          : generate(node.object);
      return `${objName}:${node.property.name}`;

    case "StringLiteral":
      return `"${node.value}"`;

    case "Identifier":
      if (node.symbolKind === "variable" || node.symbolKind === "parameter")
        return capitalize(node.name);

      return `${node.name}`;

    case "ExpressionStatement":
      return generate(node.expression);
  }
};

export default generate;
