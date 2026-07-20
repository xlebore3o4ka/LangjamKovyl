namespace Ftorlisp.OpTypes

inductive BinOp where
  | add | mul | sub | div
deriving Repr, BEq

inductive UnOp where
  | neg
deriving Repr, BEq
