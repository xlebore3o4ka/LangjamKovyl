import Lake
open Lake DSL

package "ftorlisp" where
  version := v!"0.1.0"

lean_lib «Ftorlisp» where
  -- add library configuration options here

@[default_target]
lean_exe "ftorlisp" where
  root := `Main

