# YGVM


## Описание

Виртуальная машина и язык программирования посвящённый проекту моего друга Sinopin'а.


### Идея

Идея была в том, чтобы создать виртуальную машину в которой всё представляется объектом


### Статус реализации

- `std/core`
- - [X] `gc`
- - [X] `std/core/Object`
- - - [X] `EQ` / `NEQ`
- - - [X] `hash`
- - - [X] `to_string`
- - - [X] `to_json` / `from_json`
- - [X] `std/core/Boolean`
- - - [X] `EQ`
- - - [X] `AND` / `OR` / `NOT`
- - - [ ] `XOR`
- - - [X] `to_string`
- - [X] `std/core/I64` / `std/core/F64`
- - - [X] `EQ`
- - - [X] `ADD` / `SUB` / `MUL` / `DIV` / `NEG`
- - - [X] `LT` / `LE` / `GT` / `GE`
- - - [ ] `MOD` / `POW` / `SQRT` / `AND` / `OR`
- - - [X] `to_string`
- - - [X] `to_json` / `from_json`
- - - [ ] `...`
- - [X] `std/core/String`
- - - [X] `EQ`
- - - [X] `ADD`
- - - [X] `to_i64` / `to_f64`
- - - [X] `to_json` / `from_json`
- - - [ ] `char_at` / `get_sliced`
- - - [ ] `starts_with` / `ends_with`
- - - [X] `to_string`
- - - [X] `to_json` / `from_json`
- - - [ ] `...`
- - [X] `std/core/Array`
- - - [X] `EQ`
- - - [X] `set` / `get` / `get_sliced`
- - - [X] `push` / `pop`
- - - [X] `insert` / `remove` / `remove_element`
- - - [X] `iter`
- - - [X] `to_string`
- - - [X] `to_json` / `from_json`
- - - [ ] `...`
- - [X] `std/core/ArrayIterator`
- - - [X] `has_next` / `next`
- - - [ ] `to_json` / `from_json`
- - - [ ] `...`
- - [X] `std/core/Map`
- - - [X] `EQ`
- - - [X] `set` / `get`
- - - [X] `to_string`
- - - [X] `to_json` / `from_json`
- - - [ ] `...`
- - [X] `std/core/Throwable`
- - - [ ] `stack_trace`
- - - [ ] `...`
- - [X] `std/core/Exception`
- - - [X] `to_string`
- - - [ ] `...`
- - [X] `std/core/Callable`
- - - [X] `call`
- - - [ ] `...`
- - [X] `std/core/Function`
- - - [ ] `to_json` / `from_json`


- `std/io`
- - [X] `print` / `println`
- - [X] `readline`
- - [X] `file_exits`/ `file_read` / `file_write`
- - [X] `server_socket` / `client_socket`
- - - [X] `send` / `recv`
- - - [X] `close`
- - - [ ] `...`
- - [ ] `...`


- `std/json`
- - [X] `to_string` / `from_string`


- `std/thread`
- - [X] `sleep`
- - [X] `current`
- - [X] `create`
- - [X] `std/thread/Mutex`
- - - [X] `try_with_lock` / `with_lock`
- - - [X] `try_lock` / `lock` / `unlock`
- - - [X] `set` / `get`


## Сборка

`cargo build --release`


## Запуск

На случай, если не хочется / не получается собрать в папке уже предоставлены два файла.

1. `windows` - `build/ygvm.exe`
2. `linux` - `build/ygvm`

Для запуска выполните `ygvm <file>`, если файл не указать - он запустить `examples/chat.yg`.
