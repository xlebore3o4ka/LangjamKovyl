from __future__ import annotations

import argparse
import ast
import sys
from pathlib import Path

from .compiler import compile_pyplusplus_to_python_ast, execute_pyplusplus
from .parser import parse_pyplusplus_file


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser("pypp")
    parser.add_argument("source", help="Py++ source file path")
    parser.add_argument("run_source", nargs="?", help="Optional source file to execute when using --run")
    parser.add_argument("--dump-ast", action="store_true", help="Print parsed Python AST")
    parser.add_argument("--run", action="store_true", help="Execute Py++ source")
    args, remaining_args = parser.parse_known_args(argv)
    if remaining_args and remaining_args[0] == "--":
        remaining_args = remaining_args[1:]

    source_path = Path(args.run_source if args.run and args.run_source else args.source)
    if not source_path.exists():
        print(f"File not found: {source_path}", file=sys.stderr)
        return 1

    source_ast = parse_pyplusplus_file(source_path)

    if args.dump_ast:
        compiled_ast = compile_pyplusplus_to_python_ast(source_ast, source_path=source_path)
        print(ast.dump(compiled_ast, indent=2))

    if args.run:
        original_argv = sys.argv
        try:
            sys.argv = [str(source_path)] + remaining_args
            execute_pyplusplus(source_ast, {"__name__": "__main__", "__file__": str(source_path)}, source_file=str(source_path))
        finally:
            sys.argv = original_argv
        return 0

    if not args.dump_ast:
        print(f"Parsed Py++ source from {source_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
