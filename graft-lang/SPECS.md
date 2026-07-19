# Graft-lang

Минималистичный язык программирования с акторной моделью

**Бэкенд:** трансляция в Erlang => BEAM

---

## Hello World

```graft
use std.io

fn main() {
    io.println("Hello, World!")
}
```

---

## Переменные

```graft
let x = 10
mut y = 20
y = y + 1
```

- `let` - иммутабельная привязка
- `mut` - мутабельная (транспилятор десахарит в рекурсию)

---

## Функции

```graft
fn add(a, b) {
    return a + b
}

fn greet(name) {
    io.println("Hello, " + name)
}
```

Явный `return`. Типы не аннотируются.

---

## Условия

```graft
if x > 0 {
    io.println("positive")
} else if x == 0 {
    io.println("zero")
} else {
    io.println("negative")
}
```

---

## Циклы

```graft
mut i = 0
while i < 10 {
    io.println(i)
    i = i + 1
}

let items = [1, 2, 3]
for item in items {
    io.println(item)
}
```

---

## Коллекции

```graft
let list = [1, 2, 3]
list = list.push(4)
list = list.remove(1)

let map = {"name": "Alice", "age": 30}
let name = map["name"]
```

Маппинг на Erlang: list => список, map => map.

---

## Акторы

```graft
actor ChatRoom {
    msg join(name) {
        state.users = list.push(state.users, name)
        io.println(name ++ " joined")
    }

    msg send(name, text) {
        state.messages = list.push(state.messages, [name, text])
    }

    msg history() {
        return state.messages
    }

    msg leave(name) {
        state.users = list.remove(state.users, name)
        io.println(name ++ " left")
    }
}
```

Использование:

```graft
let room = ChatRoom.spawn()
room.join("Alice")
room.send("Alice", "Hello!")
let msgs = room.history()
```

`state` - неявный параметр, транспилятор генерирует Erlang `receive`-цикл.

---

## Строки

```graft
let a = "hello"
let b = "world"
let c = a ++ b       // конкатенация: "helloworld"
let len = c.len()    // длина
```

Строки - списки символов Erlang. Оператор `+` => `++`.

---

## stdlib

| Функция | Описание |
|---------|----------|
| `io.println(x)` | Вывод с переносом |
| `io.print(x)` | Вывод без переноса |
| `list.push(lst, x)` | Добавить в конец |
| `list.remove(lst, x)` | Удалить элемент |
| `list.len(lst)` | Длина |
| `list.last(lst, n)` | Последние N элементов |

---

## Литералы

```graft
42          // целое
3.14        // дробное
true        // булево
"hello"     // строка
[1, 2, 3]  // список
{"a": 1}   // мап
```

---

## Операторы

Арифметика: `+` `-` `*` `/`
Сравнение: `==` `!=` `<` `>` `<=` `>=`
Логика: `and` `or` `not`

---

## Комментарии

```graft
// однострочный
/* многострочный */
```

---

## Ключевые слова

```
actor  and    break  continue  else   false  fn
for    if     in     let       loop   mod    msg
mut    not    or     return    spawn  state  struct
true   use
```

---

## Чат на Graft (пример)

```graft
use std.io
use std.list

actor Chat {
    msg init() {
        state.users = []
        state.messages = []
    }

    msg join(name) {
        state.users = list.push(state.users, name)
        io.println("[+] " ++ name ++ " joined")
    }

    msg send(name, text) {
        state.messages = list.push(state.messages, [name, text])
        io.println(name ++ ": " ++ text)
    }

    msg history(n) {
        return list.last(state.messages, n)
    }

    msg leave(name) {
        state.users = list.remove(state.users, name)
        io.println("[-] " ++ name ++ " left")
    }
}

fn main() {
    let chat = Chat.spawn()
    chat.join("Alice")
    chat.join("Bob")
    chat.send("Alice", "Hi Bob!")
    chat.send("Bob", "Hello Alice!")
    let msgs = chat.history(10)
    for msg in msgs {
        io.println(msg[0] ++ ": " ++ msg[1])
    }
}
```

---

## Запуск

```bash
graftc main.gft -o main.avm
atomvm main.avm atomvmlib.avm
```
