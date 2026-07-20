namespace Ftorlisp.Ty

inductive Ty where
  | number
  | bool
  | string
  | generic_cons (name : String) (arg_tys_num : Nat)
  | generic_spec (gen_cons : Ty) (arg_tys : List Ty)
  | fn (arg_tys : List Ty) (ret_ty : Ty)
  | custom (name : String)
deriving Inhabited, BEq

namespace Ty
  def tyToString : Ty → String
      | .number => "Number"
      | .string => "String"
      | .bool => "Bool"
      | .custom name => name
      | .fn arg_tys ret_ty =>
        let argsStr := "[" ++ (String.intercalate " " (arg_tys.map tyToString)) ++ "]"
        "(Fn " ++ argsStr ++  "" ++ " " ++ tyToString ret_ty ++ ")"
      | .generic_cons name _ => name
      | .generic_spec cons arg_tys =>
        let argsStr := (String.intercalate " " (arg_tys.map tyToString))
        "(" ++ (tyToString cons) ++ " " ++ argsStr ++ ")"

  instance : ToString Ty where
    toString := tyToString

  instance : Repr Ty where
    reprPrec ty _ :=
      Repr.reprPrec  (tyToString ty) 0
end Ty
