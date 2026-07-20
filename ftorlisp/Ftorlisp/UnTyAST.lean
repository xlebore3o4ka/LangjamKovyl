import Ftorlisp.OpTypes

open Ftorlisp.OpTypes

namespace Ftorlisp.UnTyAST

mutual

  inductive UnTyASTExpr where
    | number (val : Float)
    | bool (val : Bool)
    | sym (name : String)
    | string (val : String)
    | list (ty_ast : UnTyASTTy) (list : List UnTyASTExpr)
    | binOp (op : BinOp) (arg1 arg2 : UnTyASTExpr)
    | unOp (op : UnOp) (arg : UnTyASTExpr)
    | if_expr (test : UnTyASTExpr) (then_exp : UnTyASTExpr) (else_exp : UnTyASTExpr)
    | eq (args : List UnTyASTExpr) -- (args.length ≥ 2)
    | fn_call (list : List UnTyASTExpr)
    | match_exp (target : UnTyASTExpr) (branches : List (UnTyASTPattern × UnTyASTExpr))
    | first (list : UnTyASTExpr)
    | rest (list : UnTyASTExpr)
    | cons (item : UnTyASTExpr) (list : UnTyASTExpr)
  deriving Inhabited, Nonempty, Repr, BEq

  inductive UnTyASTTy where
   | sym (name : String)
   | call (name : String) (args : List UnTyASTTy)
  deriving Nonempty, Repr, BEq

  inductive UnTyASTStmt where
    | let_stmt (name : String) (val : UnTyASTExpr)
    | dec (name : String) (arg_tys : List UnTyASTTy) (ret_ty : UnTyASTTy)
    | def_stmt (name : String) (arg_names : List String) (body : List UnTyAST)
    | data_decl (name : String) (constructors : List (String × List UnTyASTTy))
  deriving Repr, BEq

  inductive UnTyAST where
    | expr (val : UnTyASTExpr)
    | stmt (val : UnTyASTStmt)
  deriving Nonempty, Repr, BEq

  inductive UnTyASTPattern where
  -- Например: (send message) -> name = "send", args = ["message"]
  | cons (name : String) (args : List String)
  | wildcard -- Для _ (игнорирование)
deriving Inhabited, Repr, BEq
end
namespace UnTyAST

end UnTyAST
end Ftorlisp.UnTyAST
