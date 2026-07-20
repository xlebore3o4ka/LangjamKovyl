-- В этом файле находится первый парсер, который переводит код в S-выражения,
-- Которые потом переводятся в абстрактное синтаксическое дерево.

import Ftorlisp.ParserCombinators
import Ftorlisp.ParseTree
open Ftorlisp.ParserCombinators
open Ftorlisp.ParseTree

namespace Ftorlisp.FirstParser

inductive FirstParserError where
  | number
  | sym
  | str
  | list
  | quote
  | unparsed
deriving Inhabited, Repr

private def fractionalPart (digits : Array Nat) : Float :=
  digits.foldr (fun d acc => (Float.ofNat d + acc) / 10.0) 0.0

private def numberParser : Parser FirstParserError ParseTree := do
  let minus ← maybe (char '-')
  let num ← wholeNumber
  let dot ← maybe $ char '.'

  let int_part := match minus with
    | .some _ => - Int.ofNat num
    | .none => num

  match dot with
    | .some _ => do
      let num_end ← many digit
      let frac := fractionalPart num_end

      return .number $ Float.ofInt num + frac
    | .none => return .number $ Float.ofInt int_part


private def isMathChar (ch : Char) : Bool :=
  ch ∈ ['+', '-', '*', '/', '=']

private def isSymbolChar (ch : Char) : Bool :=
  ch.isAlphanum || isMathChar ch || ch ∈ ['_']

private def symParser : Parser FirstParserError ParseTree := do
  let chars ← many1 (sat isSymbolChar)
  return .sym (String.ofList chars.toList)

private def isOpenBracket (char : Char) : Bool :=
  char ∈ ['(', '[']

private def isCloseBracket (char : Char) : Bool :=
  char ∈ [')', ']']

-- Состояния нашего конечного автомата
inductive StringFSMState where
  | normal
  | escape
deriving Inhabited, BEq

-- Ядро конечного автомата. Оно рекурсивно проходит по строке.
-- it - итератор строки, acc - накопленный результат, consumed - количество пройденных символов.
private partial def stringFSMCore (it : String.Legacy.Iterator) (state : StringFSMState) (acc : String) (consumed : Nat) :
    Except FirstParserError (String × Nat) :=
  if it.atEnd then
    .error FirstParserError.str -- Ошибка: дошли до конца файла, а закрывающей кавычки нет
  else
    let c := String.Legacy.Iterator.curr it
    let nextIt :=  String.Legacy.Iterator.next it
    match state with
    | .normal =>
      if c == '"' then
        .ok (acc, consumed + 1) -- Успешное завершение: встретили закрывающую кавычку
      else if c == '\\' then
        stringFSMCore nextIt .escape acc (consumed + 1) -- Переход в режим экранирования
      else
        stringFSMCore nextIt .normal (acc.push c) (consumed + 1) -- Обычный символ
    | .escape =>
      let escaped := match c with
        | 'n' => '\n'
        | 't' => '\t'
        | '"' => '"'
        | '\\' => '\\'
        | _ => c
      stringFSMCore nextIt .normal (acc.push escaped) (consumed + 1) -- Возврат в нормальный режим

-- Обертка FSM в стандартный интерфейс Parser
private def stringParser : Parser FirstParserError ParseTree :=
  fun state =>
    let inputStr := state.input
    -- Быстрая проверка: строка пустая или не начинается с кавычки?
    if inputStr.isEmpty || inputStr.front != '"' then
      .error { err := .custom FirstParserError.str, pos := state.pos, sub := .none }
    else
      -- Создаем итератор и сразу пропускаем первую открывающую кавычку
      let startIt := String.Legacy.mkIterator inputStr
           -- Запускаем автомат
      match stringFSMCore (String.Legacy.Iterator.next startIt) .normal "" 1 with
      | .ok (parsedStr, consumed) =>
        let newPos := state.pos + consumed
        -- Отбрасываем прочитанное, как это делается в твоем комбинаторе sat
        let remainingInput := (inputStr.drop consumed).toString
        -- Возвращаем ParseTree.string вместо ParseTree.sym
        .ok ⟨.string parsedStr, { input := remainingInput, pos := newPos }⟩
      | .error err =>
        .error { err := .custom err, pos := state.pos, sub := .none }

mutual
  private partial def listParser : Parser FirstParserError ParseTree := do
    let _ ←  sat isOpenBracket
    let exprs ← sepBy exprParser (many ws)
    let _ ← sat isCloseBracket
    return .call exprs.toList

  -- Парсер для 'expr -> (quote expr)
  private partial def quoteParser : Parser FirstParserError ParseTree := do
    let _ ← char '\''
    let ty_expr ← exprParser
    let quoted_list ← listParser -- Рекурсивно парсим следующее выражение
    match quoted_list with
      | .call val => return .list ty_expr val
      | _ => panic! "quote приняла не список"

  private partial def exprParser : Parser FirstParserError ParseTree := do
    (withErr (.custom .number)  numberParser) <|>
    (withErr (.custom FirstParserError.sym)  symParser) <|>
    (withErr (.custom FirstParserError.list) listParser) <|>
    (withErr (.custom FirstParserError.str) stringParser) <|>
    (withErr (.custom FirstParserError.quote) quoteParser)

end

def eof : Parser ε Unit :=
  fun state =>
    if state.input.isEmpty then
      .ok ⟨(), state⟩
    else
      .error { err := .inputNotReduce, pos := state.pos, sub := .none }

partial def exprFirstParser (src : String) : Except FirstParserError ParseTree :=
  match exprParser ⟨src, 0⟩ with
    | .error err => match err.err with
      | .custom fperr => .error fperr
      | _ => unreachable!
    | .ok parser_res => .ok parser_res.val

partial def programParser : Parser FirstParserError (List ParseTree) := do
  let _ ← maybe $ many ws
  let arr ← sepBy exprParser (many ws)
  let _ ← maybe $ many ws -- Съедаем пробелы после последнего выражения
  let _ ← withErr (.custom FirstParserError.unparsed) eof -- Требуем конец строки
  return arr.toList

partial def programFirstParser (src : String) : Except FirstParserError (List ParseTree) :=
  match programParser ⟨src, 0⟩ with
    | .error err => match err.err with
      | .custom fperr => .error fperr
      | _ => unreachable!
    | .ok parser_res => .ok parser_res.val

#eval (exprFirstParser "(+ 1 2 \"12\" (* 3 4))")
