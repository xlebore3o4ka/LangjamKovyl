from __future__ import annotations

import ast as py_ast
import io
import tokenize as py_tokenize
from dataclasses import dataclass
from enum import Enum
from typing import Union

KEYWORDS = {
    "if",
    "else",
    "elif",
    "while",
    "for",
    "in",
    "is",
    "not",
    "and",
    "or",
    "func",
    "when",
    "macro",
    "use",
    "as",
    "from",
    "class",
    "extends",
    "return",
    "pass",
    "with",
    "break",
    "continue",
    "match",
    "case",
    "yield",
    "True",
    "False",
    "None",
    "raise",
    "assert",
    "try",
    "except",
    "finally",
    "del",
    "global",
    "nonlocal",
    "async",
    "await",
}

SINGLE_CHAR_TOKENS = {
    ";": "SEMICOLON",
    "{": "LBRACE",
    "}": "RBRACE",
    ",": "COMMA",
    "(": "LPAREN",
    ")": "RPAREN",
    "[": "LBRACKET",
    "]": "RBRACKET",
    ".": "DOT",
    ":": "COLON",
    "$": "DOLLAR",
    "@": "AT",
}


class TokenKind(str, Enum):
    NAME = "NAME"
    NUMBER = "NUMBER"
    STRING = "STRING"
    FSTRING = "FSTRING"
    KEYWORD = "KEYWORD"
    OP = "OP"
    SEMICOLON = "SEMICOLON"
    LBRACE = "LBRACE"
    RBRACE = "RBRACE"
    COMMA = "COMMA"
    LPAREN = "LPAREN"
    RPAREN = "RPAREN"
    LBRACKET = "LBRACKET"
    RBRACKET = "RBRACKET"
    DOT = "DOT"
    COLON = "COLON"
    DOLLAR = "DOLLAR"
    AT = "AT"
    NEWLINE = "NEWLINE"
    EOF = "EOF"

type TokenValue = Union[str, int, float, bool, None]

@dataclass(frozen=True)
class Token:
    kind: TokenKind
    value: TokenValue
    line: int
    column: int


def tokenize(source: str) -> list[Token]:
    tokens: list[Token] = []
    line_number = 1
    column = 1
    position = 0
    length = len(source)

    while position < length:
        char = source[position]

        if char in " \t\r":
            position += 1
            column += 1
            continue
        if char == "\n":
            position += 1
            line_number += 1
            column = 1
            continue
        if char == "#" and position + 1 < length and source[position + 1] == "#":
            while position < length and source[position] != "\n":
                position += 1
            continue
        if char == "#" and position + 1 < length and source[position + 1] == "*":
            position += 2
            column += 2
            while position < length:
                if source[position:position + 2] == "*#":
                    position += 2
                    column += 2
                    break
                if source[position] == "\n":
                    position += 1
                    line_number += 1
                    column = 1
                    continue
                position += 1
                column += 1
            continue
        if _starts_string(source, position):
            token_text, new_position, new_line, new_column = _read_string(source, position, line_number, column)
            prefix = token_text[: token_text.find(token_text.lstrip('rRuUfFbB')[0])]
            is_fstring = any(ch.lower() == 'f' for ch in prefix)
            if is_fstring:
                tokens.append(Token(TokenKind.FSTRING, token_text, line_number, column))
            else:
                try:
                    value = py_ast.literal_eval(token_text)
                except Exception as exc:
                    raise SyntaxError(f"Invalid string literal at line {line_number}, col {column}: {exc}") from exc
                tokens.append(Token(TokenKind.STRING, value, line_number, column))
            position = new_position
            line_number = new_line
            column = new_column
            continue
        if char.isalpha() or char == "_":
            start = position
            while position < length and (source[position].isalnum() or source[position] == "_"):
                position += 1
            value = source[start:position]
            kind = TokenKind.KEYWORD if value in KEYWORDS else TokenKind.NAME
            tokens.append(Token(kind, value, line_number, column))
            column += position - start
            continue
        if char.isdigit():
            start = position
            value, position = _read_number(source, position)
            tokens.append(Token(TokenKind.NUMBER, value, line_number, column))
            column += position - start
            continue
        three_char = source[position : position + 3]
        if three_char in {"//=", "**=", "<<=", ">>="}:
            tokens.append(Token(TokenKind.OP, three_char, line_number, column))
            position += 3
            column += 3
            continue
        two_char = source[position : position + 2]
        if two_char in {"==", "!=", "<=", ">=", "<<", ">>", "&&", "||", "//", "**", "++", "--", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "->"}:
            tokens.append(Token(TokenKind.OP, two_char, line_number, column))
            position += 2
            column += 2
            continue
        if char in SINGLE_CHAR_TOKENS:
            tokens.append(Token(TokenKind[SINGLE_CHAR_TOKENS[char]], char, line_number, column))
            position += 1
            column += 1
            continue
        if char in "+-*/%<>!=&|^~" and char != ":":
            tokens.append(Token(TokenKind.OP, char, line_number, column))
            position += 1
            column += 1
            continue
        raise SyntaxError(f"Unexpected character {char!r} on line {line_number}, col {column}")

    tokens.append(Token(TokenKind.EOF, None, line_number, column))
    return tokens


def _starts_string(source: str, position: int) -> bool:
    if position >= len(source):
        return False
    if source[position] in "'\"":
        return True
    prefix = ""
    i = position
    while i < len(source) and source[i].lower() in {"r", "u", "b", "f"}:
        prefix += source[i]
        i += 1
    if not prefix:
        return False
    if i >= len(source):
        return False
    return source[i] in "'\""

def _read_string(source: str, start: int, line_number: int, column: int) -> tuple[str, int, int, int]:
    substring = source[start:]
    reader = io.StringIO(substring).readline
    token_gen = py_tokenize.generate_tokens(reader)
    try:
        token = next(token_gen)
        while token.type in {py_tokenize.ENCODING, py_tokenize.NL, py_tokenize.NEWLINE}:
            token = next(token_gen)
    except StopIteration:
        raise SyntaxError(f"Unterminated string literal starting at line {line_number}, col {column}")

    if token.type == py_tokenize.STRING:
        token_text = token.string
        end_line, end_col = token.end
    elif token.type == py_tokenize.FSTRING_START:
        end_line = token.end[0]
        end_col = token.end[1]
        while True:
            try:
                token = next(token_gen)
            except StopIteration:
                raise SyntaxError(f"Unterminated f-string literal starting at line {line_number}, col {column}")
            if token.type == py_tokenize.FSTRING_END:
                end_line, end_col = token.end
                break
        token_text = substring[: sum(len(line) for line in substring.splitlines(True)[: end_line - 1]) + end_col]
    else:
        raise SyntaxError(f"Invalid string literal start at line {line_number}, col {column}")

    lines = substring.splitlines(True)
    if end_line - 1 < len(lines):
        new_position = start + sum(len(line) for line in lines[: end_line - 1]) + end_col
    else:
        new_position = start + len(substring)
    new_line = line_number + end_line - 1
    new_column = end_col + 1 if end_line > 1 else column + end_col
    return token_text, new_position, new_line, new_column

def _read_number(text: str, start: int) -> tuple[int | float, int]:
    position = start
    while position < len(text) and text[position].isdigit():
        position += 1
    if position < len(text) and text[position] == ".":
        position += 1
        if position >= len(text) or not text[position].isdigit():
            raise SyntaxError(f"Invalid number literal at column {start + 1}")
        while position < len(text) and text[position].isdigit():
            position += 1
        return float(text[start:position]), position
    return int(text[start:position]), position
