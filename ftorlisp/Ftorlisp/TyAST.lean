import Ftorlisp.Ty
import Ftorlisp.OpTypes

open Ftorlisp.OpTypes
open Ftorlisp.Ty

namespace Ftorlisp.TyAST
mutual
  inductive TyASTExpr where
    | number (ty : Ty) (val : Float)
    | bool (ty : Ty) (val : Bool)
    | string (ty : Ty) (val : String)
    | list (ty : Ty) (list : List TyASTExpr)
    | binOp (ty : Ty) (op : BinOp) (arg1 arg2 : TyASTExpr)
    | unOp (ty : Ty) (op : UnOp) (arg : TyASTExpr)
    | varRead (ty : Ty) (name : String)
    | if_expr (ty : Ty) (test : TyASTExpr) (then_exp : TyASTExpr) (else_exp : TyASTExpr)
    | eq (ty : Ty) (args : List TyASTExpr)
    | fn_expr (ty : Ty) (list : List TyASTExpr)
    | match_exp (ty : Ty) (target : TyASTExpr) (branches : List (TyASTPattern × TyASTExpr))
    | first (ty : Ty) (list : TyASTExpr)
    | rest (ty : Ty) (list : TyASTExpr)
    | cons (ty : Ty) (item : TyASTExpr) (list : TyASTExpr)
  deriving Inhabited, BEq

  inductive TyASTStmt where
    | let_stmt (ty : Ty) (name : String) (val : TyASTExpr)
    | dec (ty : Ty) (name : String)
    | def_stmt (ty : Ty)
      (name : String) (arg_names : List String) (body : List TyAST)
    | data_decl (ty : Ty) (name : String) (constructors : List (String × List Ty))
  deriving Inhabited, BEq

  inductive TyAST where
    | exp (val : TyASTExpr)
    | stmt (val : TyASTStmt)
  deriving Inhabited, BEq

  inductive TyASTPattern where
  | cons (ty : Ty) (name : String) (args : List (String × Ty))
  | wildcard (ty : Ty)
deriving Inhabited, BEq

end

namespace TyASTExpr
  def ty (ast: TyASTExpr) : Ty :=
    match ast with
      | .number ty _ => ty
      | .bool ty _ => ty
      | .varRead ty _ => ty
      | .string ty _ => ty
      | .list ty _ => ty
      | .binOp ty _ _ _ => ty
      | .unOp ty _ _ => ty
      | .eq ty _ => ty
      | .if_expr ty _ _ _ => ty
      | .fn_expr ty _ => ty
      | .match_exp ty _ _ => ty
      | .first ty _ => ty
      | .rest ty _ => ty
      | .cons ty _ _ => ty

end TyASTExpr

namespace Ftorlisp.TyASTPrinter

-- Вспомогательная функция для генерации нужного количества пробелов
def spaces (n : Nat) : String :=
  String.ofList (List.replicate n ' ')

-- Печать паттерна match-выражения.
-- Не является взаимно рекурсивной функцией, поэтому вынесена из mutual блока.
def patternToString (pat : TyASTPattern) : String :=
  match pat with
  | .wildcard ty =>
    s!"_ : {Ty.tyToString ty}"
  | .cons ty name args =>
    if args.isEmpty then
      s!"({name}) : {Ty.tyToString ty}"
    else
      let argsStr := String.intercalate " " (args.map (fun (n, t) => s!"({n} : {Ty.tyToString t})"))
      s!"({name} {argsStr}) : {Ty.tyToString ty}"


mutual
  -- Обработка выражений
  partial def exprToString (ast : TyASTExpr) (ind : Nat := 0) : String :=
    match ast with
    | .number ty val =>
      s!"{val} : {Ty.tyToString ty}"
    | .bool ty val =>
      s!"{val} : {Ty.tyToString ty}"
    | .varRead ty name =>
      s!"{name} : {Ty.tyToString ty}"
    | .string ty string =>
    s!"{string} : {Ty.tyToString ty}"
    | .list ty list => let indNext := ind + 2
      let itemsStrs := list.map (fun e => exprToString e indNext)
      let body := String.intercalate s!"\n{spaces indNext}" itemsStrs
      s!"'[{body}] : {Ty.tyToString ty}"
    | .unOp ty op arg =>
      let opS := match op with | .neg => "-"
      -- Длина префикса: "(" + "op" + " " (например, "(- " это 3 символа)
      let indNext := ind + 2 + opS.length
      s!"({opS} {exprToString arg indNext}) : {Ty.tyToString ty}"
    | .binOp ty op arg1 arg2 =>
      let opS := match op with
        | .add => "+"
        | .sub => "-"
        | .mul => "*"
        | .div => "/"
      -- Длина префикса: "(" + "op" + " "
      let indNext := ind + 2 + opS.length
      s!"({opS} {exprToString arg1 indNext}\n{spaces indNext}{exprToString arg2 indNext}) : {Ty.tyToString ty}"
    | .if_expr ty test then_exp else_exp =>
      -- Длина префикса: "(if " (4 символа)
      let indNext := ind + 4
      s!"(if {exprToString test indNext}\n{spaces indNext}{exprToString then_exp indNext}\n{spaces indNext}{exprToString else_exp indNext}) : {Ty.tyToString ty}"

    | .eq ty args =>
      -- Длина префикса: "(eq " (4 символа)
      let indNext := ind + 4
      let argsStrs := args.map (fun e => exprToString e indNext)
      let body := String.intercalate s!"\n{spaces indNext}" argsStrs
      s!"(eq {body}) : {Ty.tyToString ty}"
    | .fn_expr ty list =>
      -- Для вызова функции отступ для аргументов равен 2 символам (сразу под именем функции, учитывая "(" )
      let indNext := ind + 2
      let argsStrs := list.map (fun e => exprToString e indNext)
      let body := String.intercalate s!"\n{spaces indNext}" argsStrs
      s!"({body}) : {Ty.tyToString ty}"
    | .match_exp ty target branches =>
      -- Длина префикса: "(match " (7 символов) — под ним печатается target
      let targetIndent := ind + 7
      let targetStr := exprToString target targetIndent
      -- Ветки печатаются с отступом ind + 2
      let branchesIndent := ind + 2
      -- Тело каждой ветки печатается с отступом branchesIndent + 2 (учитывая "[")
      let bodyIndent := branchesIndent + 2
      let branchStrs := branches.map (fun (pat, e) =>
        let patStr := patternToString pat
        let bodyStr := exprToString e bodyIndent
        s!"[{patStr}\n{spaces bodyIndent}{bodyStr}]")
      let branchesJoined := String.intercalate s!"\n{spaces branchesIndent}" branchStrs
      if branches.isEmpty then
        s!"(match {targetStr}) : {Ty.tyToString ty}"
      else
        s!"(match {targetStr}\n{spaces branchesIndent}{branchesJoined}) : {Ty.tyToString ty}"
    | .first ty list =>
      let indNext := ind + 7
      s!"(first {exprToString list indNext}) : {Ty.tyToString ty}"
    | .rest ty list =>
      let indNext := ind + 6
      s!"(rest {exprToString list indNext}) : {Ty.tyToString ty}"
    | .cons ty item list =>
      let indNext := ind + 6
      s!"(cons {exprToString item indNext}\n{spaces indNext}{exprToString list indNext}) : {Ty.tyToString ty}"

  -- Обработка утверждений (statements)
  partial def stmtToString (ast : TyASTStmt) (ind : Nat := 0) : String :=
    match ast with
    | .let_stmt _ty name val =>
      -- Длина префикса: "(let " (5) + имя переменной + " " (1)
      let indNext := ind + 6 + name.length
      s!"(let {name} {exprToString val indNext})"
    | .dec ty name =>
      -- Для dec нам нужно разобрать тип функции, чтобы получить список аргументов и возвращаемый тип
      match ty with
      | .fn arg_tys ret_ty =>
        let argsStr := "[" ++ String.intercalate " " (arg_tys.map Ty.tyToString) ++ "]"
        s!"(dec {name} {argsStr} {Ty.tyToString ret_ty})"
      | _ =>
        -- Fallback, если dec по какой-то причине имеет не функциональный тип
        s!"(dec {name} : {Ty.tyToString ty})"
    | .def_stmt ty name arg_names body =>
      -- Формируем список аргументов в квадратных скобках
      let argsStr := "[" ++ String.intercalate " " arg_names ++ "]"
      -- Стандартный отступ для тела функции — 2 пробела относительно текущего
      let indNext := ind + 2
      let bodyStrs := body.map (fun a => astToString a indNext)

      if bodyStrs.isEmpty then
        s!"(def {name} {argsStr}) : {Ty.tyToString ty}"
      else
        let bodyJoined := String.intercalate s!"\n{spaces indNext}" bodyStrs
        s!"(def {name} {argsStr}\n{spaces indNext}{bodyJoined}) : {Ty.tyToString ty}"
    | .data_decl _ _ _=> "ty decl not implemented print"

  -- Главная функция для перевода всего узла AST
  partial def astToString (ast : TyAST) (ind : Nat := 0) : String :=
    match ast with
    | .exp e => exprToString e ind
    | .stmt s => stmtToString s ind
end
end Ftorlisp.TyASTPrinter

instance : Repr TyAST where
    reprPrec ast _ :=
      Repr.reprPrec  (Ftorlisp.TyASTPrinter.astToString ast) 0

instance : Repr TyASTExpr where
    reprPrec ast _ :=
      Repr.reprPrec  (Ftorlisp.TyASTPrinter.exprToString ast) 0

instance : Repr TyASTStmt where
    reprPrec ast _ :=
      Repr.reprPrec  (Ftorlisp.TyASTPrinter.stmtToString ast) 0
