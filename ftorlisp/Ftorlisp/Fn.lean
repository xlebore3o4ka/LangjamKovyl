import Ftorlisp.Ty
import Ftorlisp.TyAST

open Ftorlisp.Ty
open Ftorlisp.TyAST

namespace Ftorlisp.Fn

structure FnDef where
  ast : TyASTStmt -- ast : TyASTStmt.def_stmt
deriving Inhabited, Repr, BEq

inductive FnError where
  | fnDefined
  | incorrectArgsNum
  | tyIsNotFn
deriving Repr, BEq

structure Fn where
  ty : Ty
  definition : Option FnDef
deriving Inhabited, Repr, BEq

namespace Fn
  def make (ty : Ty) : Except FnError Fn :=
    match ty with
      | .fn _ _ => .ok ⟨ty, .none⟩
      | _ => .error .tyIsNotFn

  def makeFromDecTy (dec_ty : Ty) : Fn :=
    ⟨dec_ty, .none⟩

  def addDef (fn : Fn) (fn_def : FnDef) : Except FnError Fn :=
    match fn.definition with
      | .some _ => .error .fnDefined
      | _ => .ok {fn with definition := .some fn_def}
end Fn
