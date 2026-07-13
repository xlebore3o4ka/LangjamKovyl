import fs from "fs";
import path from "path";
import peggy from "peggy";

const main = () => {
  const filePath = process.argv[2];

  if (!filePath) {
    console.error("Error: path to .clx file not specified");
    process.exit(1);
  }

  if (!fs.existsSync(filePath)) {
    console.error(`Error: not found file: ${filePath}`);
    process.exit(1);
  }

  try {
    const sourceCode = fs.readFileSync(filePath, "utf-8");

    const grammarPath = path.resolve("src/grammar.peggy");
    const grammar = fs.readFileSync(grammarPath, "utf-8");

    const parser = peggy.generate(grammar);

    const ast: string = parser.parse(sourceCode);

    console.log(JSON.stringify(ast, null, 2));

    console.log(sourceCode);
  } catch (error: unknown) {
    console.error(`Error: fail to read file: ${filePath}`, error);
    process.exit(1);
  }
};

main();
