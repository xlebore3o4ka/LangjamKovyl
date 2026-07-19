# The Graft Language
Простенький ЯП на конкурс([ТЫК](conds.md)). Настоящая реализация компилятора транспилирует код в Erlang, а затем, компилятор `erlc` компилирует выхлоп в beam байткод. 

```C#
use std.io

fn main() {
    io.println("Hello, World!")
}

```

## Зависимости
### для сборки:
- go 1.26.5

### runtime
- Erlang Compiler (OTP 21+)
- Стандартная библиотека Erlang: gen_tcp, maps, lists, io_lib

### для подключения к чату
- telnet или что-то такое. по вкусу

## Сама сборка:
```
$ make install # сборка + прокид в $PATH директорию. если нужна просто сборка - make build
```

## Запуск сервера чата:
```
$ graftc examples/chat.gft --run
graftc: examples/chat.gft -> chat.erl
graftc: chat.erl -> chat.beam

Chat server started on port 4000
Use <telnet localhost 4000> command to join
```

---

[_Условия конкурсе_](conds.md), [_Спецификация Graft_](SPECS.md), [_Код чата_](examples/chat.gft)