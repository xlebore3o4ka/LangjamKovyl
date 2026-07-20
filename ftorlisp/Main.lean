import Ftorlisp.FirstParser
import Ftorlisp.SecondParser
import Ftorlisp.TyInference
import Ftorlisp.Context
import Ftorlisp.TyAST
import Ftorlisp.Codegen

open Ftorlisp.FirstParser
open Ftorlisp.SecondParser
open Ftorlisp.TyInference
open Ftorlisp.Context
open Ftorlisp.TyAST

inductive GeneralError where
  | firstParserError (err :  FirstParserError)
  | secondParserError (err : SecondParserError)
  | tyInfError (err : TyInfError)
deriving Repr

partial def srcToTyAST (src : String) (context : Context) :
    Except GeneralError $ (List TyAST × Context) := do
  let pt ← programFirstParser src |> Except.mapError GeneralError.firstParserError
  let utast ← programSecondParser pt |> Except.mapError GeneralError.secondParserError
  let tyast ← blockTyInference utast context |> Except.mapError GeneralError.tyInfError
  return tyast

namespace Ftorlisp.Driver

-- ==========================================================================
-- Опции компиляции, собираемые из аргументов командной строки
-- ==========================================================================

structure CompilerOptions where
  inputPath    : String
  outputPath   : Option String := none
  moduleName   : Option String := none
  -- Заголовки: только dec, реализация — на стороне Erlang. Код не генерируется.
  stdlibPaths  : List String := []
  -- Prelude: реальный Ftorlisp-код, компилируется вместе с пользовательским.
  preludePaths : List String := []
  -- Если true — печатаем результат в stdout вместо записи в файл.
  emitOnly     : Bool := false
deriving Inhabited

-- ==========================================================================
-- Утилиты
-- ==========================================================================

/-- Извлекает "имя модуля" из пути к файлу: путь/до/foo.ftl -> foo -/
def defaultModuleNameFromPath (path : String) : String :=
  let base := (path.splitOn "/").getLastD path
  (base.splitOn ".").headD base

def readFileEx (path : String) : IO (Except String String) := do
  try
    let content ← IO.FS.readFile path
    return .ok content
  catch e =>
    return .error s!"Не удалось прочитать файл {path}: {e}"

def formatError (path : String) (err : GeneralError) : String :=
  match err with
  | .firstParserError e =>
    s!"Ошибка синтаксического анализа (этап 1) в файле {path}:\n{toString (repr e)}"
  | .secondParserError e =>
    s!"Ошибка синтаксического анализа (этап 2) в файле {path}:\n{toString (repr e)}"
  | .tyInfError e =>
    s!"Ошибка проверки типов в файле {path}:\n{toString (repr e)}"

-- ==========================================================================
-- Загрузка отдельных файлов
-- ==========================================================================

/-- Обрабатывает файл только ради пополнения `Context`.
    Полученный `TyAST` отбрасывается — реализация уже есть в Erlang. -/
def loadDeclsOnly (path : String) (context : Context) :
    IO (Except String Context) := do
  match ← readFileEx path with
  | .error e => return .error e
  | .ok src =>
    match srcToTyAST src context with
    | .error e => return .error (formatError path e)
    | .ok (_ast, ctx') => return .ok ctx'

/-- Полная обработка файла: парсинг + тайпчекинг, с возвратом AST и контекста. -/
def loadAndTypecheck (path : String) (context : Context) :
    IO (Except String (List TyAST × Context)) := do
  match ← readFileEx path with
  | .error e => return .error e
  | .ok src =>
    match srcToTyAST src context with
    | .error e => return .error (formatError path e)
    | .ok res => return .ok res

-- ==========================================================================
-- Последовательная загрузка списков stdlib/prelude файлов
-- ==========================================================================

partial def loadStdlibs (paths : List String) (ctx : Context) :
    IO (Except String Context) := do
  match paths with
  | [] => return .ok ctx
  | p :: rest =>
    match ← loadDeclsOnly p ctx with
    | .error e => return .error e
    | .ok ctx' => loadStdlibs rest ctx'

partial def loadPreludes (paths : List String) (ctx : Context) (acc : List TyAST) :
    IO (Except String (List TyAST × Context)) := do
  match paths with
  | [] => return .ok (acc, ctx)
  | p :: rest =>
    match ← loadAndTypecheck p ctx with
    | .error e => return .error e
    | .ok (ast, ctx') => loadPreludes rest ctx' (acc ++ ast)

-- ==========================================================================
-- Полный пайплайн компиляции
-- ==========================================================================

def compile (opts : CompilerOptions) : IO (Except String String) := do
  -- 1. Стартовый контекст.
  --    ЗАМЕЧАНИЕ: предполагается наличие Context.empty. Если в вашей
  --    реализации Context строится иначе — замените эту строку.
  let initialContext : Context := Context.init

  -- 2. Заголовки (--stdlib): только контекст, без кодогена.
  match ← loadStdlibs opts.stdlibPaths initialContext with
  | .error e => return .error e
  | .ok ctxAfterStdlib =>

    -- 3. Prelude (--prelude): контекст + AST для последующей склейки в кодоген.
    match ← loadPreludes opts.preludePaths ctxAfterStdlib [] with
    | .error e => return .error e
    | .ok (preludeAst, ctxAfterPrelude) =>

      -- 4. Основной пользовательский файл.
      match ← loadAndTypecheck opts.inputPath ctxAfterPrelude with
      | .error e => return .error e
      | .ok (userAst, _finalCtx) =>

        -- 5. Склеиваем prelude + пользовательский код в единый модуль.
        let combinedAst := preludeAst ++ userAst

        -- 6. Имя модуля и генерация кода.
        let moduleName := opts.moduleName.getD (defaultModuleNameFromPath opts.inputPath)
        let erlangCode := Ftorlisp.Codegen.compileModule moduleName combinedAst

        return .ok erlangCode

-- ==========================================================================
-- Разбор аргументов командной строки
-- ==========================================================================

def usage : String :=
  "Использование: ftorlisp <файл.ftl> [опции]\n" ++
  "Опции:\n" ++
  "  -o <файл.erl>       Путь для выходного файла (по умолчанию <модуль>.erl)\n" ++
  "  --module <имя>      Имя генерируемого Erlang-модуля\n" ++
  "  --stdlib <файл.ftl> Заголовок: только dec, реализация в Erlang (можно " ++
                          "указывать несколько раз)\n" ++
  "  --prelude <файл.ftl> Реальный Ftorlisp-код, компилируется вместе с " ++
                          "программой (можно указывать несколько раз)\n" ++
  "  --print              Печатать результат в stdout вместо записи в файл\n"

partial def parseArgsGo (args : List String) (opts : CompilerOptions) :
    Except String CompilerOptions :=
  match args with
  | [] =>
    if opts.inputPath.isEmpty then
      .error usage
    else
      .ok opts
  | "-o" :: path :: rest =>
    parseArgsGo rest { opts with outputPath := some path }
  | "--module" :: name :: rest =>
    parseArgsGo rest { opts with moduleName := some name }
  | "--stdlib" :: path :: rest =>
    parseArgsGo rest { opts with stdlibPaths := opts.stdlibPaths ++ [path] }
  | "--prelude" :: path :: rest =>
    parseArgsGo rest { opts with preludePaths := opts.preludePaths ++ [path] }
  | "--print" :: rest =>
    parseArgsGo rest { opts with emitOnly := true }
  | path :: rest =>
    if opts.inputPath.isEmpty then
      parseArgsGo rest { opts with inputPath := path }
    else
      .error s!"Неожиданный аргумент: {path}\n\n{usage}"

def parseArgs (args : List String) : Except String CompilerOptions :=
  if args.isEmpty then
    .error usage
  else
    parseArgsGo args {
      inputPath := "", outputPath := none, moduleName := none,
      stdlibPaths := [], preludePaths := [], emitOnly := false
    }

def resolveOutputPath (opts : CompilerOptions) (moduleName : String) : String :=
  opts.outputPath.getD (moduleName ++ ".erl")

end Ftorlisp.Driver

-- ==========================================================================
-- Точка входа
-- ==========================================================================

open Ftorlisp.Driver in
def main (args : List String) : IO UInt32 := do
  match parseArgs args with
  | .error msg =>
    IO.eprintln msg
    return 1
  | .ok opts =>
    match ← compile opts with
    | .error msg =>
      IO.eprintln s!"Ошибка компиляции:\n{msg}"
      return 1
    | .ok erlangCode =>
      if opts.emitOnly then
        IO.println erlangCode
        return 0
      else
        let moduleName := opts.moduleName.getD (defaultModuleNameFromPath opts.inputPath)
        let outPath := resolveOutputPath opts moduleName
        try
          IO.FS.writeFile outPath erlangCode
          IO.println s!"Скомпилировано: {opts.inputPath} -> {outPath}"
          return 0
        catch e =>
          IO.eprintln s!"Не удалось записать файл {outPath}: {e}"
          return 1
