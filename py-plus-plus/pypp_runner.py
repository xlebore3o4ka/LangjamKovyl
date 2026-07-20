from __future__ import annotations

import argparse
import sys
from pathlib import Path

from pyplusplus.compiler import compile_pyplusplus_to_python_ast, execute_pyplusplus
from pyplusplus.parser import parse_pyplusplus_file


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser("pypp-runner")
    parser.add_argument("source", help="Py++ source file path")
    parser.add_argument("--dump-ast", action="store_true", help="Print compiled Python AST")
    parser.add_argument("--run", action="store_true", help="Execute the Py++ source")
    parser.add_argument("--args", nargs=argparse.REMAINDER, help="Arguments passed to the executed Py++ script")
    args = parser.parse_args(argv)

    source_path = Path(args.source)
    if not source_path.exists():
        print(f"File not found: {source_path}", file=sys.stderr)
        return 1

    source_ast = parse_pyplusplus_file(source_path)

    if args.dump_ast:
        compiled_ast = compile_pyplusplus_to_python_ast(source_ast, source_path=source_path)
        print(compiled_ast)

    if args.run:
        original_argv = sys.argv
        try:
            sys.argv = [str(source_path)] + (args.args or [])
            execute_pyplusplus(source_ast, {"__name__": "__main__", "__file__": str(source_path)}, source_file=str(source_path))
        finally:
            sys.argv = original_argv
        return 0

    print(f"Parsed Py++ source from {source_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
