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

**Chat for LangJam**

```kovyl
func clear() do
  # "Чистит" консоль
  int i = 0
  while i < 100 do 
    print:(fmt:"")
    i = i + 1 
  end
end

func help() do
  print:fmt:"Доступные команды:"
  print:fmt:"  /stop     - остановка работы чата"
  print:fmt:"  /help     - справка"
  print:fmt:"  /members  - список участников"
  print:fmt:"  /add      - добавить участника"
  print:fmt:"  /remove   - удалить участника"
  print:fmt:"  /behalf   - переключаться между участниками"
end

func char[*] input(char[*] prompt) do
  print:(prompt, term="")
  return read:()
end

func int index(char[*][*] array, char[*] str) do
  int idx = 0
  for el = array do
    if el == str do return idx end
    idx = idx + 1
  end
  return -1
end

print:fmt:"\nДобро пожаловать в чат\n#! KovylCHAT !#\n"
help()

char[*] current = arr:"Admin"
char[*][*] members = arr:{current};
(char[*] name, char[*] message)[*] history = arr:{(
	name = arr:"System", 
	message = arr:"Chat opened!"
)}

while true do
  char[*] msg = input(fmt:('\n', current, " > "))

  clear()

  for data = history do
  	print:fmt:(data.name, ": ", data.message)
  end

  print:fmt:("\n", current, " > ", msg)

  if msg == "/stop" do break

  elif msg == "/help" do 
  	help()

  elif msg == "/members" do 
    for member = members do
      print:fmt:("- ", member)
    end

  elif msg == "/add" do 
	  char[*] name = input(fmt:"  name > ")
	  
	  if name == "System" do
	    print:fmt:"  Error: 'System' is a reserved name"
	    continue
	  end
	  
	  if index(members, name) != -1 do
	    print:fmt:("  Error: member '", name, "' already exists")
	    continue
	  end
	  
	  int last = len:(members)
	  resize:(members, last + 1)
	  members[last] = name

  elif msg == "/remove" do
    char[*] name = input(fmt:"  name > ")
    int idx = index(members, name)
    
    if idx == -1 do
      print:fmt:"  Error: unknown member"
      continue
    end
    
    int i = idx
    while i < len:(members) - 1 do
      members[i] = members[i + 1]
      i = i + 1
    end
    
    resize:(members, len:(members) - 1)
    
    if current == name do
      current = members[0]
    end

  elif msg == "/behalf" do 
  	char[*] name = input(fmt:"  name > ")

  	if index(members, name) == -1 do
  	  print:fmt:"  Error: unknown member"
      continue
  	end

  	current = name
  else do
  	int last = len:(history)
  	resize:(history, last + 1)
  	history[last] = (name = current, message = msg)
  end
end
```

**Run**
`nimble run -- chat.kvl`

Yep, you need nim for this🤗
