import Ftorlisp.UnTyAST
import Ftorlisp.TyAST
import Ftorlisp.OpTypes
import Ftorlisp.Ty
import Ftorlisp.Context

open Ftorlisp.OpTypes
open Ftorlisp.UnTyAST
open Ftorlisp.TyAST
open Ftorlisp.Ty
open Ftorlisp.Context

namespace Ftorlisp.TyInference

inductive TyInfError where
  | undefinedVar (name : String)
  | arithArgsTypeMismatch (arg1 arg2 : TyASTExpr)
  | arithNoArgs
  | negNotNum (arg : TyASTExpr)
  | ifTypeMissmatch (then_ast : TyASTExpr) (else_ast : TyASTExpr)
  | ifConditionNotBool (test : TyASTExpr)
  | eqArgsLess2 (args :  List TyASTExpr)
  | eqTyMismatch (args :  List TyASTExpr)
  | unknownTy (unty_ast : UnTyASTTy)
  | varDefined (name : String)
  | genericArgsNumMismatch (unty_ast : UnTyASTTy) (correct_num : Nat)
  | genericFirstNotCons (unty_ast : UnTyASTTy)
  | decAlreadyDeclared (ty_ast : TyAST)
  | fnTyMismatch (oper : TyASTExpr) (args : List TyASTExpr)
  | fnOperNotFn (oper : TyASTExpr)
  | fnBadArgsNum (oper : TyASTExpr) (args : List TyASTExpr)
  | defStmtFnDefined (name : String)
  | defStmtFnUndeclaered (name : String)
  | defBadArgsNum (stmt : UnTyASTStmt)
  | defLastNotExpr (stmt : UnTyASTStmt)
  | defRetTyMismatch (stmt : UnTyASTStmt)
  | listTyMismatch (expr : UnTyASTExpr)
  -- Новые ошибки для match
  | matchEmptyBranches
  | matchBranchTyMismatch (expected : Ty) (actual : Ty)
  | matchUnknownCons (name : String)
  | matchConsNotFn (name : String)
  | matchConsTyMismatch (cons_name : String) (expected_ty : Ty)
  | matchConsArgsNumMismatch (cons_name : String)
  | firstNotList (ty : Ty)
  | restNotList (ty : Ty)
  | consNotList (ty : Ty)
  | consTypeMismatch (item_ty : Ty) (list_ty : Ty)
deriving Repr, BEq

abbrev TyInfExcept := Except TyInfError

mutual
  private partial def expTyInference
    (exp : UnTyASTExpr) (context : Context) : TyInfExcept TyASTExpr :=
    match exp with
      | .number val => .ok $ .number context.tyNumber val
      | .bool val => .ok $ .bool context.tyBool val
      | .string val => .ok $ .string context.tyString val
      | .list ty_ast list => do
        let ty ← tyTyInference ty_ast context
        let list_asts ← list.mapM (expTyInference · context)
        if list_asts.all (·.ty == ty) then
          return .list (context.tyListMake ty) list_asts
        else
          .error $ .listTyMismatch exp

      | .sym name => match (context.varTyLookup name) with
        | .some ty => .ok $ .varRead ty name
        | .none => .error $ .undefinedVar name

      | .binOp op arg1 arg2 => do
        let arg1_ast ← expTyInference arg1 context
        let arg2_ast ← expTyInference arg2 context

        if arg1_ast.ty == arg2_ast.ty && arg1_ast.ty == context.tyNumber then
          return .binOp arg1_ast.ty op arg1_ast arg2_ast
        else
          .error $ .arithArgsTypeMismatch arg1_ast arg2_ast

      | .unOp .neg arg => do
        let arg_ast ← expTyInference arg context

        if arg_ast.ty == context.tyNumber then
          return (.unOp arg_ast.ty .neg arg_ast)
        else
          .error $ .negNotNum arg_ast

      | .if_expr test then_exp else_exp => do
        let test_ast ← expTyInference test context
        let then_ast ← expTyInference then_exp context
        let else_ast ← expTyInference else_exp context

        match test_ast.ty with
          | .bool =>
            if then_ast.ty == else_ast.ty then
              return .if_expr then_ast.ty test_ast then_ast else_ast
            else
              .error $ .ifTypeMissmatch then_ast else_ast
          | _ => .error $ .ifConditionNotBool test_ast
      | .eq args => do
        let args_tyasts ← args.mapM (expTyInference · context)
        match args_tyasts with
          | [] | [_] => .error $ .eqArgsLess2 args_tyasts
          | first :: rest => do
            if rest.all (·.ty == first.ty) then
              return .eq context.tyBool args_tyasts
            else
              .error $ .eqTyMismatch args_tyasts

      | .fn_call (oper :: args) => fnCallTyInferenct oper args context
      | .fn_call [] => unreachable!
      -- ...
      | .match_exp target branches => do
        let target_ast ← expTyInference target context

        if branches.isEmpty then
          .error .matchEmptyBranches
        else
          -- Функция для обработки одной ветки
          let inferBranch (branch : UnTyASTPattern × UnTyASTExpr) (expected_ret_ty_opt : Option Ty) : TyInfExcept (TyASTPattern × TyASTExpr) := do
            -- Обязательно делаем levelUp, чтобы изолировать переменные ветки!
            let (pat_ast, branch_ctx) ← patternTyInference branch.1 target_ast.ty (context.levelUp)
            let body_ast ← expTyInference branch.2 branch_ctx

            match expected_ret_ty_opt with
              | .some expected_ret_ty =>
                if body_ast.ty == expected_ret_ty then
                  return (pat_ast, body_ast)
                else
                  .error $ .matchBranchTyMismatch expected_ret_ty body_ast.ty
              | .none => return (pat_ast, body_ast)

          -- Тип первой ветки диктует тип всего выражения
          let first_branch_ast ← inferBranch branches.head! none
          let ret_ty := first_branch_ast.2.ty

          -- Проверяем остальные ветки
          let rest_branches_asts ← branches.tail!.mapM (inferBranch · (some ret_ty))

          return .match_exp ret_ty target_ast (first_branch_ast :: rest_branches_asts)
      | .first list => do
        let list_ast ← expTyInference list context
        match getListElemTy list_ast.ty with
          | .some elem_ty => return .first elem_ty list_ast
          | .none => .error $ .firstNotList list_ast.ty

      | .rest list => do
        let list_ast ← expTyInference list context
        match getListElemTy list_ast.ty with
          | .some _ => return .rest list_ast.ty list_ast
          | .none => .error $ .restNotList list_ast.ty

      | .cons item list => do
        let item_ast ← expTyInference item context
        let list_ast ← expTyInference list context
        match getListElemTy list_ast.ty with
          | .some elem_ty =>
            if item_ast.ty == elem_ty then
              return .cons list_ast.ty item_ast list_ast
            else
              .error $ .consTypeMismatch item_ast.ty list_ast.ty
          | .none => .error $ .consNotList list_ast.ty
      -- ...
  partial def getListElemTy (ty : Ty) : Option Ty :=
    match ty with
    | .generic_spec (.generic_cons "List" 1) [elem_ty] => .some elem_ty
    | _ => .none

  private partial def patternTyInference
    (pat : UnTyASTPattern)
    (expected_ty : Ty)
    (context : Context) : TyInfExcept (TyASTPattern × Context) := do
    match pat with
      | .wildcard => return (.wildcard expected_ty, context)
      | .cons name arg_names => do
        let cons_fn_opt := context.fnLookup name
        match cons_fn_opt with
          | .some cons_fn =>
            match cons_fn.ty with
              | .fn arg_tys ret_ty =>
                -- Проверяем, что конструктор действительно от того типа, который мы матчим
                if ret_ty == expected_ty then
                  if arg_names.length == arg_tys.length then
                    -- Рекурсивно добавляем переменные из паттерна в контекст
                    let rec loop (ctx : Context) (names : List String) (tys : List Ty) (acc : List (String × Ty)) : TyInfExcept (Context × List (String × Ty)) :=
                      match names, tys with
                        | n :: ns, t :: ts => do
                          let (new_ctx, is_success) := ctx.varTyInsert n t
                          if !is_success then
                            .error $ .varDefined n
                          else
                            loop new_ctx ns ts (acc ++ [(n, t)])
                        | [], [] => .ok (ctx, acc)
                        | _, _ => .error $ .matchConsArgsNumMismatch name

                    let (new_context, bound_args) ← loop context arg_names arg_tys []
                    return (.cons expected_ty name bound_args, new_context)
                  else
                    .error $ .matchConsArgsNumMismatch name
                else
                  .error $ .matchConsTyMismatch name expected_ty
              | _ => .error $ .matchConsNotFn name
          | .none => .error $ .matchUnknownCons name

  private partial def fnCallTyInferenct
    (oper : UnTyASTExpr)
    (args : List UnTyASTExpr)
    (context : Context): TyInfExcept TyASTExpr := do

    let oper_ty_ast ← (expTyInference oper context)
    let arg_tys_asts ← args.mapM (expTyInference · context)
    let actual_tys := arg_tys_asts.map (·.ty)
    match oper_ty_ast.ty with
      | .fn expexted_tys ret_ty => do
        if expexted_tys.length == actual_tys.length then
          let eq_list := List.zipWith (· == ·) expexted_tys actual_tys
          if eq_list.all (·) then
            return .fn_expr ret_ty (oper_ty_ast :: arg_tys_asts)
          else
            .error $ .fnTyMismatch oper_ty_ast arg_tys_asts
        else
          .error $ .fnBadArgsNum oper_ty_ast arg_tys_asts
      | _ => .error $ .fnOperNotFn oper_ty_ast



  private partial def tyTyInference
    (ty_ast : UnTyASTTy) (context : Context) : TyInfExcept Ty := do
    match ty_ast with
      | .sym name => match context.tyLookup name with
        | .some ty => return ty
        | .none => .error $ .unknownTy ty_ast
      | .call name arg_tys_asts =>
        let opt_ty_cons := context.tyLookup name
        match opt_ty_cons with
          | .some ty_cons => match ty_cons with
            | .generic_cons _name arg_tys_num => do
              if arg_tys_asts.length == arg_tys_num then
                let arg_tys ← arg_tys_asts.mapM (tyTyInference · context)
                return .generic_spec ty_cons arg_tys
              else
                .error $ .genericArgsNumMismatch ty_ast arg_tys_num
            | _ => .error $ .genericFirstNotCons ty_ast
          | .none => .error $ .unknownTy ty_ast

  private partial def stmtTyInference
    (stmt : UnTyASTStmt) (context : Context) : TyInfExcept TyASTStmt := do
    match stmt with
      | .let_stmt name val => do
        let val_ast ← expTyInference val context
        return (.let_stmt val_ast.ty name val_ast)
      | .dec name arg_tys_asts ret_ty_ast => do
        let args_tys ← arg_tys_asts.mapM (tyTyInference · context)
        let ret_ty ← (tyTyInference ret_ty_ast context)
        return .dec (.fn args_tys ret_ty) name
      | .def_stmt name args body => do
        let dec_opt := context.fnLookup name
        match dec_opt with
          | .none => .error $ .defStmtFnUndeclaered name
          | .some ⟨.fn arg_tys ret_ty, .none⟩  => do
            let nested_context := context.levelUp
            let correct_num := arg_tys.length
            let actual_num := args.length

            if correct_num != actual_num then
              .error $ .defBadArgsNum stmt

            let rec loop (context : Context) (names : List String) (tys : List Ty) : TyInfExcept Context :=
              match names, tys with
                | name :: names_rest, ty :: tys_rest => do
                  let (new_context, is_success) := context.varTyInsert name ty
                  if !is_success then
                    .error $ .varDefined name
                  loop new_context names_rest tys_rest
                | [], [] => .ok context
                | _, _ => .error $ .defBadArgsNum stmt

            let new_context ← loop nested_context args arg_tys
            let (typed_body, _) ← blockTyInference body new_context

            let last := typed_body.getLast!
            match last with
              | .exp last_exp => do
                if last_exp.ty == ret_ty then
                  let dec := dec_opt.get!
                  return .def_stmt dec.ty name args typed_body
              | _ => .error $ .defLastNotExpr stmt

            .error $ .defRetTyMismatch stmt
          | _ => panic! "В декларации тип не функция"
      | .data_decl name constructors => do
        let ty := Ty.custom name
        let typed_constructors ← constructors.mapM (fun (cons_name, arg_tys_asts) => do
          let arg_tys ← arg_tys_asts.mapM (tyTyInference · context)
          return (cons_name, arg_tys)
        )
        return .data_decl ty name typed_constructors
      -- ...

  partial def astTyInference
    (ast : UnTyAST) (context : Context) : TyInfExcept TyAST := do
    match ast with
      | .expr exp => do
        let exp_typed ← expTyInference exp context
        return .exp exp_typed
      | .stmt stmt => do
        let stmt_typed ← stmtTyInference stmt context
        return .stmt stmt_typed

  partial def blockTyInference
    (ast_list : List UnTyAST) (context : Context) : TyInfExcept $ (List TyAST × Context) := do
    match ast_list with
      | [] => return ([], context)
      | ast :: rest => do
        let tyast ← astTyInference ast context
        match tyast with
          | .exp _ => do
            let (rest_tyast, rest_context) ← (blockTyInference rest context)

            return (tyast :: rest_tyast, rest_context)
          | .stmt stmt => do
            match stmt with
              | .let_stmt ty name _ => do
                let (new_context, is_success) := context.varTyInsert name ty
                if !is_success then
                  .error $ .varDefined name

                let (rest_tyast, rest_context) ← (blockTyInference rest new_context)

                return (tyast :: rest_tyast, rest_context)
              | .dec ty name => do
                let fn := Fn.Fn.makeFromDecTy ty
                let (new_context, is_success) := context.fnInsert name fn
                if !is_success then
                  .error $ .decAlreadyDeclared tyast
                else
                  let (rest_tyast, rest_context) ← (blockTyInference rest new_context)
                  return (tyast :: rest_tyast, rest_context)
              | .def_stmt _ty name _arg_names _body => do
                let new_context := context.fnAddDef name ⟨stmt⟩
                match new_context with
                  | .error _ => .error $ .defStmtFnDefined name
                  | .ok new_context_ok => do
                    let (rest_tyast, rest_context) ← (blockTyInference rest new_context_ok)
                    return (tyast :: rest_tyast, rest_context)
              -- ...
              | .data_decl ty name constructors => do
                -- 1. Добавляем новый тип в контекст
                let ctx_with_ty := context.tyInsert name ty

                -- 2. Вспомогательная функция для добавления конструкторов
                let rec insertCons (ctx : Context) (cons_list : List (String × List Ty)) : TyInfExcept Context :=
                  match cons_list with
                  | [] => .ok ctx
                  | (cons_name, arg_tys) :: rest_cons => do
                    -- Конструктор — это функция, возвращающая наш новый тип
                    let cons_ty := .fn arg_tys ty
                    let fn := Fn.Fn.makeFromDecTy cons_ty
                    let (new_ctx, is_success) := ctx.fnInsert cons_name fn

                    if !is_success then
                      .error $ .decAlreadyDeclared tyast -- Можно переиспользовать эту ошибку
                    else
                      insertCons new_ctx rest_cons

                let new_context ← insertCons ctx_with_ty constructors

                -- 3. Продолжаем проверку остального блока с обновленным контекстом
                let (rest_tyast, rest_context) ← (blockTyInference rest new_context)
                return (tyast :: rest_tyast, rest_context)
              -- ...

end
