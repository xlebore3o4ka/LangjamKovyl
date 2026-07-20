import Ftorlisp.Ty
import Ftorlisp.OpTypes
import Ftorlisp.TyAST

open Ftorlisp.OpTypes
open Ftorlisp.Ty
open Ftorlisp.TyAST

namespace Ftorlisp.Codegen

-- ==========================================================================
-- Таблица extern-функций: имя в Ftorlisp -> квалифицированный вызов Erlang.
-- Если функция объявлена через (dec ...) без (def ...) и её имя есть в этой
-- таблице — компилятор эмитит вызов Module:Function(...) вместо локального.
-- ==========================================================================

def externTable : List (String × String) := [
  ("print",          "stdlib:print"),
  ("println",        "stdlib:println"),
  ("print_num",      "stdlib:print_num"),
  ("println_num",    "stdlib:println_num"),

  ("lt",  "stdlib:lt"),
  ("gt",  "stdlib:gt"),
  ("lte", "stdlib:lte"),
  ("gte", "stdlib:gte"),
  ("neq", "stdlib:neq"),

  ("ft_mod",  "stdlib:ft_mod"),
  ("ft_abs",  "stdlib:ft_abs"),
  ("ft_min",  "stdlib:ft_min"),
  ("ft_max",  "stdlib:ft_max"),
  ("ft_pow",  "stdlib:ft_pow"),
  ("ft_sqrt", "stdlib:ft_sqrt"),

  ("str_concat",   "stdlib:str_concat"),
  ("str_len",      "stdlib:str_len"),
  ("str_upper",    "stdlib:str_upper"),
  ("str_lower",    "stdlib:str_lower"),
  ("str_trim",     "stdlib:str_trim"),
  ("str_contains", "stdlib:str_contains"),

  ("number_to_string",  "stdlib:number_to_string"),
  ("string_to_number",  "stdlib:string_to_number"),
  ("bool_to_string", "stdlib:bool_to_string"),

  ("length",   "stdlib:ft_list_length"),
  ("is_empty", "stdlib:ft_list_is_empty"),
  ("reverse",  "stdlib:ft_list_reverse"),
  ("append",   "stdlib:ft_list_append"),
  ("nth",      "stdlib:ft_list_nth"),
  ("str_eq", "stdlib:str_eq"),
  ("read_line", "stdlib:read_line"),
  ("str_split_once", "stdlib:str_split_once"),
  ("str_list_take", "stdlib:str_list_take"),
  ("str_list_remove", "stdlib:str_list_remove"),
  ("str_list_contains", "stdlib:str_list_contains"),
  ("str_eq", "stdlib:str_eq"),
  ("read_line", "stdlib:read_line"),
  ("str_split_once", "stdlib:str_split_once"),
  ("str_list_take", "stdlib:stdlib:str_list_take"),
  ("str_list_remove", "stdlib:str_list_remove"),
  ("str_list_contains", "stdlib:str_list_contains")
]

def lookupExtern (name : String) : Option String :=
  (externTable.find? (fun p => p.fst == name)).map (fun p => p.snd)

-- ==========================================================================
-- Ключевые слова Erlang. Если после мангла имя совпадает с одним из них,
-- добавляем суффикс "_", чтобы не сломать компиляцию сгенерированного кода.
-- ==========================================================================

def erlangReservedWords : List String := [
  "after", "and", "andalso", "band", "begin", "bnot", "bor", "bsl", "bsr",
  "bxor", "case", "catch", "cond", "div", "end", "fun", "if", "let", "not",
  "of", "or", "orelse", "query", "receive", "rem", "try", "when", "xor"
]

def avoidReserved (s : String) : String :=
  if erlangReservedWords.contains s then s ++ "_" else s

-- ==========================================================================
-- Манглинг имён
-- ==========================================================================

def isValidIdentChar (c : Char) : Bool :=
  c.isAlpha || c.isDigit || c == '_'

def mangleChars (s : String) : String :=
  String.mk (s.toList.map (fun c => if isValidIdentChar c then c else '_'))

/-- Имя функции/атома Erlang должно начинаться с маленькой буквы. -/
def mangleAtomName (name : String) : String :=
  let cleaned := mangleChars name
  let result :=
    match cleaned.toList with
    | [] => "empty_atom"
    | c :: cs =>
      if c.isUpper then
        String.mk (c.toLower :: cs)
      else if c.isDigit || c == '_' then
        String.mk ('a' :: c :: cs)
      else
        cleaned
  avoidReserved result

/-- Имя переменной Erlang должно начинаться с большой буквы (или '_'). -/
def mangleVarName (name : String) : String :=
  let cleaned := mangleChars name
  match cleaned.toList with
  | [] => "Empty_var"
  | c :: cs =>
    if c.isLower then
      String.mk (c.toUpper :: cs)
    else if c.isDigit then
      String.mk ('V' :: c :: cs)
    else
      cleaned

-- ==========================================================================
-- Литералы
-- ==========================================================================

/-- Числа в AST хранятся как Float, поэтому в Erlang всегда генерируем
    float-литерал. Если нужны "настоящие" целые в рантайме — используйте
    trunc/1 в самой stdlib (что мы уже сделали для int_to_string и т.п.). -/
def numToErlang (val : Float) : String :=
  toString val

def boolToErlang (b : Bool) : String :=
  if b then "true" else "false"

def escapeErlangChar (c : Char) : String :=
  match c with
  | '"'  => "\\\""
  | '\\' => "\\\\"
  | '\n' => "\\n"
  | '\t' => "\\t"
  | '\r' => "\\r"
  | c    => String.singleton c

def escapeErlangString (s : String) : String :=
  "\"" ++ String.join (s.toList.map escapeErlangChar) ++ "\""

def spaces (n : Nat) : String :=
  String.ofList (List.replicate n ' ')

-- ==========================================================================
-- Окружение кодогенерации.
-- topConsts — топ-уровневые (let ...), ставшие функциями арности 0.
-- localFuns — имена вложенных (def ...), которые стали локальными
--             рекурсивными fun'ами и вызываются как переменные.
-- ==========================================================================

structure Env where
  topConsts : List String
  localFuns : List String
  ctors : List String
deriving Inhabited

def collectConstructors (program : List TyAST) : List String :=
  program.flatMap (fun item =>
    match item with
    | .stmt (.data_decl _ _ ctors) => ctors.map (fun (cname, _) => cname)
    | _ => [])

-- ==========================================================================
-- Паттерны match
-- ==========================================================================

def patternToErlang (pat : TyASTPattern) : String :=
  match pat with
  | .wildcard _ => "_"
  | .cons _ name args =>
    let tag := mangleAtomName name
    if args.isEmpty then
      s!"\{{tag}}" -- Оборачиваем в кортеж даже пустые конструкторы
    else
      let argsStr := String.intercalate ", " (args.map (fun (n, _) => mangleVarName n))
      "{" ++ tag ++ ", " ++ argsStr ++ "}"

-- ==========================================================================
-- Цепочка равенств для (eq a b c ...)
-- ==========================================================================

partial def eqChainErlang : List String → String
  | [] => "true"
  | [_] => "true"
  | a :: b :: rest =>
    let head := s!"({a} =:= {b})"
    match rest with
    | [] => head
    | _  => head ++ " andalso " ++ eqChainErlang (b :: rest)

-- ==========================================================================
-- Компиляция выражений
-- ==========================================================================

partial def exprToErlang (env : Env) (ast : TyASTExpr) (ind : Nat := 4) : String :=
  match ast with
  | .number _ val =>
    numToErlang val
  | .bool _ val =>
    boolToErlang val
  | .string _ val =>
    escapeErlangString val
  | .varRead _ name =>
    if env.ctors.contains name then
      s!"\{{mangleAtomName name}}" -- Генерируем кортеж {cmd_quit}
    else if env.topConsts.contains name then
      s!"{mangleAtomName name}()"
    else
      mangleVarName name
  | .list _ items =>
    let itemsStr := String.intercalate ", " (items.map (fun e => exprToErlang env e ind))
    "[" ++ itemsStr ++ "]"
  | .unOp _ op arg =>
    match op with
    | .neg => s!"(- {exprToErlang env arg ind})"
  | .binOp _ op arg1 arg2 =>
    let opS := match op with
      | .add => "+"
      | .sub => "-"
      | .mul => "*"
      | .div => "/"
    s!"({exprToErlang env arg1 ind} {opS} {exprToErlang env arg2 ind})"
  | .if_expr _ test then_exp else_exp =>
    let indNext := ind + 4
    let testStr := exprToErlang env test ind
    let thenStr := exprToErlang env then_exp indNext
    let elseStr := exprToErlang env else_exp indNext
    s!"(case {testStr} of\n{spaces indNext}true -> {thenStr};\n{spaces indNext}false -> {elseStr}\n{spaces ind}end)"
  | .eq _ args =>
    let strs := args.map (fun e => exprToErlang env e ind)
    "(" ++ eqChainErlang strs ++ ")"
  | .fn_expr _ items =>
    match items with
    | [] => "ok"
    | head :: args =>
      let argsStr := String.intercalate ", " (args.map (fun e => exprToErlang env e ind))
      match head with
      | .varRead _ name =>
        if env.ctors.contains name then
          -- Это вызов конструктора типа!
          let tag := mangleAtomName name
          if args.isEmpty then
            s!"\{{tag}}" -- Генерируем кортеж вместо голого атома
          else
            "{" ++ tag ++ ", " ++ argsStr ++ "}"
        else if env.localFuns.contains name then
          s!"{mangleVarName name}({argsStr})"
        else match lookupExtern name with
          | some qualified => s!"{qualified}({argsStr})"
          | none => s!"{mangleAtomName name}({argsStr})"
      | other =>
        -- Вызов вычисленного функционального значения: (Expr)(Args)
        s!"({exprToErlang env other ind})({argsStr})"
  | .match_exp _ target branches =>
    let indNext := ind + 4
    let targetStr := exprToErlang env target ind
    let branchStrs := branches.map (fun (pat, body) =>
      let patStr := patternToErlang pat
      let bodyStr := exprToErlang env body indNext
      s!"{spaces indNext}{patStr} -> {bodyStr}")
    let branchesJoined := String.intercalate ";\n" branchStrs
    s!"(case {targetStr} of\n{branchesJoined}\n{spaces ind}end)"
  | .first _ list =>
    s!"hd({exprToErlang env list ind})"
  | .rest _ list =>
    s!"tl({exprToErlang env list ind})"
  | .cons _ item list =>
    s!"[{exprToErlang env item ind} | {exprToErlang env list ind}]"

-- ==========================================================================
-- Компиляция тела (List TyAST) в последовательность Erlang-операторов.
-- В Erlang выражение "X = Expr" само по себе возвращает значение Expr,
-- поэтому дополнительной обработки "последнего выражения" не требуется:
-- если тело заканчивается на let, оно всё равно вернёт нужное значение.
-- ==========================================================================

partial def bodyToErlang (env : Env) (items : List TyAST) (ind : Nat := 4) : String :=
  let rec go (env : Env) (items : List TyAST) (acc : List String) : List String :=
    match items with
    | [] => acc.reverse
    | (.exp e) :: rest =>
      go env rest (exprToErlang env e ind :: acc)
    | (.stmt (.dec _ _)) :: rest =>
      -- dec не порождает кода — это только сигнатура для тайпчекера
      go env rest acc
    | (.stmt (.data_decl _ _ _)) :: rest =>
      -- вложенные объявления типов не поддерживаются, пропускаем
      go env rest acc
    | (.stmt (.let_stmt _ name val)) :: rest =>
      let line := s!"{mangleVarName name} = {exprToErlang env val ind}"
      go env rest (line :: acc)
    | (.stmt (.def_stmt _ name arg_names body)) :: rest =>
      let fname := mangleVarName name
      let argsStr := String.intercalate ", " (arg_names.map mangleVarName)
      let innerEnv := { env with localFuns := name :: env.localFuns }
      let indNext := ind + 4
      let bodyStr := bodyToErlang innerEnv body indNext
      let line := s!"{fname} = fun {fname}({argsStr}) ->\n{spaces indNext}{bodyStr}\n{spaces ind}end"
      -- имя видно и последующим операторам того же тела (let*-семантика)
      go innerEnv rest (line :: acc)
  let stmts := go env items []
  if stmts.isEmpty then
    "ok"
  else
    String.intercalate s!",\n{spaces ind}" stmts

-- ==========================================================================
-- Сбор информации о топ-уровне модуля
-- ==========================================================================

def collectTopLevelConstNames (program : List TyAST) : List String :=
  program.filterMap (fun item =>
    match item with
    | .stmt (.let_stmt _ name _) => some name
    | _ => none)

def collectTopLevelFuncArities (program : List TyAST) : List (String × Nat) :=
  program.filterMap (fun item =>
    match item with
    | .stmt (.def_stmt _ name arg_names _) => some (name, arg_names.length)
    | _ => none)

def collectDataDecls (program : List TyAST) : List (String × List (String × List Ty)) :=
  program.filterMap (fun item =>
    match item with
    | .stmt (.data_decl _ name ctors) => some (name, ctors)
    | _ => none)

-- ==========================================================================
-- Компиляция целого модуля
-- ==========================================================================

def compileModule (moduleName : String) (program : List TyAST) : String :=
  let topConsts := collectTopLevelConstNames program
  let topFuncs  := collectTopLevelFuncArities program
  let ctors     := collectConstructors program
  let env : Env := { topConsts := topConsts, localFuns := [], ctors := ctors }

  -- ---- заголовок и экспорт ----
  let constExports := topConsts.map (fun n => s!"{mangleAtomName n}/0")
  let funcExports  := topFuncs.map (fun (n, arity) => s!"{mangleAtomName n}/{arity}")
  let allExports   := constExports ++ funcExports ++ ["main/0"]
  let exportsStr   := String.intercalate ", " allExports
  let header := s!"-module({mangleAtomName moduleName}).\n-export([{exportsStr}]).\n"

  -- ---- комментарии для алгебраических типов ----
  let dataComments := (collectDataDecls program).map (fun (name, ctors) =>
    let ctorsStr := String.intercalate ", " (ctors.map (fun (cname, ctys) =>
      if ctys.isEmpty then
        cname
      else
        cname ++ "(" ++ String.intercalate ", " (ctys.map Ty.tyToString) ++ ")"))
    s!"%% data {name}: {ctorsStr}")
  let dataCommentsStr :=
    if dataComments.isEmpty then "" else String.intercalate "\n" dataComments ++ "\n"

  -- ---- топ-уровневые константы -> функции арности 0 ----
  let constDefs := program.filterMap (fun item =>
    match item with
    | .stmt (.let_stmt _ name val) =>
      let fname := mangleAtomName name
      let valStr := exprToErlang env val 4
      some (s!"{fname}() ->\n    {valStr}.\n")
    | _ => none)
  let constDefsStr := String.intercalate "\n" constDefs

  -- ---- функции верхнего уровня ----
  let funcDefs := program.filterMap (fun item =>
    match item with
    | .stmt (.def_stmt _ name arg_names body) =>
      let fname := mangleAtomName name
      let argsStr := String.intercalate ", " (arg_names.map mangleVarName)
      let bodyStr := bodyToErlang env body 4
      some (s!"{fname}({argsStr}) ->\n    {bodyStr}.\n")
    | _ => none)
  let funcDefsStr := String.intercalate "\n" funcDefs

  -- ---- main/0: побочные эффекты в порядке появления в модуле ----
  let mainStmts := program.filterMap (fun item =>
    match item with
    | .exp e => some (exprToErlang env e 4)
    | .stmt (.let_stmt _ name _) => some (s!"{mangleAtomName name}()")
    | _ => none)
  let mainBody :=
    if mainStmts.isEmpty then "ok" else String.intercalate ",\n    " mainStmts
  let mainFunc := s!"main() ->\n    {mainBody}.\n"

  String.intercalate "\n" ([header, dataCommentsStr, constDefsStr, funcDefsStr, mainFunc].filter (· != ""))

end Ftorlisp.Codegen
