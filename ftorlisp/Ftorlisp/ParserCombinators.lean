namespace Ftorlisp.ParserCombinators

abbrev ParserPos := Nat


structure ParserState where
  input : String
  pos : ParserPos
deriving Repr, BEq

structure ParserRes (α : Type) where
  val : α
  state : ParserState
deriving Repr, BEq

inductive ErrorVal (ε : Type) where
  | mismatch
  | endOfInput
  | conversionFail
  | inputNotReduce
  | custom (val : ε)
deriving Repr, BEq

structure ParserError (ε : Type) where
  err : ErrorVal ε
  pos : ParserPos
  sub : Option (ParserError ε)
deriving Repr, BEq

def Parser (ε α : Type) := ParserState → Except (ParserError ε) (ParserRes α)

def ppure (val : α): Parser ε α :=
  λ state => .ok ⟨val, state⟩

def pbind (pars : Parser ε α) (fn : α -> Parser ε β): Parser ε β :=
  λ state => match pars state with
    | .error err => .error err
    | .ok ⟨val, state⟩ => (fn val) state

def pfail (err : ErrorVal ε) : Parser ε α :=
  λ state => .error { err := err, pos := state.pos, sub := .none }

-- Превращает Option в Parser. Если .none, падает с указанной ошибкой.
def fromOption (opt : Option α) (err : ErrorVal ε) : Parser ε α :=
  match opt with
  | .some val => ppure val
  | .none     => pfail err

instance : Monad (Parser ε) where
  pure := ppure
  bind := pbind


def sat (pred : Char → Bool) : Parser ε Char :=
  λ state =>
    let str := state.input
    if (str.isEmpty) then
      let pos := state.pos
      .error { err := .endOfInput, pos := pos, sub := .none}
    else
      if pred (str.front) then
        let new_input := (str.drop 1).toString
        let new_pos := state.pos + 1
        .ok ⟨str.front, { input := new_input, pos := new_pos }⟩
      else
        .error { err := .mismatch, pos := state.pos, sub := .none}

partial def manyCore (parser : Parser ε α) (acc : Array α): Parser ε (Array α) :=
  λ state =>
    match parser state with
      | .error _ => .ok ⟨acc, state⟩
      | .ok ⟨val, new_state⟩ => manyCore parser (acc.push val) new_state
      -- Если парсер не уменьшает длину строки, manyCore уйдёт в бесконечный цикл.

partial def many (parser : Parser ε α): Parser ε (Array α) :=
  λ state => (manyCore parser #[]) state

partial def many1 (parser : Parser ε α): Parser ε (Array α) :=
  λ state => match (parser state) with
    | .error err => .error { err := .mismatch, pos := state.pos, sub := err}
    | .ok _ => many parser state

instance : OrElse (Parser ε α) where
  orElse parser_1 unit_to_parser :=  λ state =>
    let res := parser_1 state
    match res with
      | .ok val => .ok val
      | .error err =>
        let res2 := unit_to_parser () state
        match res2 with
          | .ok _ => res2
          | .error err2 =>
            if err.pos > err2.pos then
              .error err
            else
              .error err2

instance : Functor (Parser ε) where
  map f parser := λ state =>
    let res := parser state
    match res with
      | .ok ⟨val, new_state⟩ => .ok ⟨f val, new_state⟩
      | .error err => .error err

def maybe (parser : Parser ε α) : Parser ε (Option α) :=
  λ state =>
    let res := parser state
    match res with
      | .ok ⟨val, new_state⟩ => .ok ⟨.some val, new_state⟩
      | .error _ => .ok ⟨.none, state⟩

def withSep (parser : Parser ε α) (separator : Parser ε β) : Parser ε α := do
  let res ← parser
  let _ ← maybe separator
  return res

def withErr (err : ErrorVal ε) (parser : Parser ε α) : Parser ε α :=
  λ state =>
    let res := parser state
    match res with
      | .ok _ => res
      | .error sub_err => .error ⟨err, sub_err.pos, sub_err⟩

def sepBy (parser : Parser ε α) (separator : Parser ε β) : Parser ε (Array α) := do
  many (withSep parser separator)


def char (ch : Char) : Parser ε Char :=
  sat (· = ch)

def string (pattern : String) : Parser ε String :=
  λ state =>
    if (state.input.startsWith pattern) then
      let rest := (state.input.drop (pattern.length)).toString
      .ok ⟨pattern, ⟨rest, state.pos + pattern.length⟩⟩
    else
      .error ⟨.mismatch, state.pos, .none⟩

def ws : Parser ε Char :=
  sat (·.isWhitespace)


def digitToNat (ch : Char): Option Nat :=
  let code := ch.toNat
  if (47 < code) && (code < 58) then
    .some (code - 48)
  else
    .none

def digit : Parser ε Nat := do
  let ch ← sat Char.isDigit
  fromOption (digitToNat ch) .conversionFail


-- Этот комбинатор парсит набор цифр, которые идут подряд друг за другом,
-- без разделения при помощи знака _
def wholeNumber : Parser ε Nat := do
  let digits ← many1 digit
  let num := digits.foldl (fun acc num => acc * 10 + num) 0
  return num

#guard let parser : Parser Unit Nat := digit
  match (parser ⟨"1", 0⟩) with
    | .ok ⟨num, _⟩ => num = 1
    | .error _ => false

#guard let parser : Parser Unit Nat := digit
  match (parser ⟨"1", 0⟩) with
    | .ok ⟨num, _⟩ => num = 1
    | .error _ => false
