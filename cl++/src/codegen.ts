import stdlib from "./stdlib.js";
import { type ASTNode } from "./types/index.js";

const capitalize = (str: string): string =>
  str.charAt(0).toUpperCase() + str.slice(1);

const generate = (node: ASTNode): string => {
  switch (node.type) {
    case "Program":
      const moduleHeader = `-module(main).\n-export([main/0]).\n\n`;
      const body = node.body.map(generate).join("\n\n");
      return moduleHeader + body;

    case "FunctionDeclaration":
      const funcName = node.name.name;
      const stmts = node.body.map(generate).join(",\n    ");
      return `${funcName}() ->\n    ${stmts}.`;

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
