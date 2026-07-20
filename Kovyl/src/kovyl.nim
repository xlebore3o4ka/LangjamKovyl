import core/[parser, astnodes, errors]
import utils/[strerr]
import std/[os]
import core/visitors/[SemanticAnalyzerVisitor, InterpreterVisitor]

proc main() =
  let args = commandLineParams()
  
  if args.len == 0:
    stderr.writeLine("Usage: ", getAppFilename(), " <file>")
    quit(1)
  
  var shortErrors = false
  var showAST = false
  var debug = false
  var filePath = ""
  
  for arg in args:
    if arg == "-s":
      shortErrors = true
    if arg == "-a":
      showAST = true
    if arg == "-d":
      debug = true
    else:
      filePath = arg
  
  if filePath.len == 0:
    stderr.writeLine("Usage: ", getAppFilename(), " [-t] [-r] <file>")
    quit(1)
  
  if not fileExists(filePath):
    stderr.writeLine("Error: File not found: ", filePath)
    quit(1)
  
  let text = readFile(filePath)

  stdout.writeLine("")

  var parser = newParser(text, filePath)
  var blockStatement: BlockStatement = parser.parse()

  semanticAnalyzerLogging(false)
  interpreterVisitorLogging(false)
  
  if errors.errors.len == 0:
    if debug:
      semanticAnalyzerLogging(true)
    newSemanticAnalyzerVisitor().visitStatement(blockStatement)
  
  # if showAST:
  #   echo newASTPrinterVisitor().printStatement(blockStatement)

  if errors.errors.len == 0:
    if debug:
      echo "[KOVYL] INFO: Compilation successful!"
      semanticAnalyzerLogging(false)
      interpreterVisitorLogging(true)
      
    let interpreter = newInterpreterVisitor()
    interpreter.visitStatement(blockStatement)

    if debug:
      echo "\n[KOVYL] INFO: Running successful!"
    interpreterVisitorLogging(false)

  for error in errors.errors:
    printError(error, shortErrors)

when isMainModule:
  main()