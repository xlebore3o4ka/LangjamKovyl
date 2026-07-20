namespace Ftorlisp.ParseTree

inductive ParseTree where
  | number (val : Float)
  | sym (name : String)
  | string (val : String)
  | call (list : List ParseTree)
  | list (ty_expr : ParseTree) (list : List ParseTree)
deriving Inhabited, Nonempty, Repr, BEq
