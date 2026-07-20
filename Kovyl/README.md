# KOVYL

**KOVYL** is a statically typed programming language with manual memory management.

The language emphasizes explicitness of operations — with no hidden runtime behavior or garbage collector.

KOVYL offers a rich type system, functions as first-class objects, array operations, as well as utilities for string manipulation and formatting. Both procedural and functional programming paradigms are supported.

The language syntax is designed to be readable and unambiguous.

**Syntax example:**

```kovyl
func char[*] greeting(char[32] name) do 
  return fmt:("Hello from Kovyl, ", name, "!")
end

char[32][] names = {"Alice", "Ben", "John"}

for name = names do
  print:(greeting(name), free=true)
end
```
