# ФторЛисп
Фторлисп - это функциональный, статически типизированный Лисп.

Описание языка находится в spec.org

ОНО ЗАПУСКАЕТСЯ!!!
```bash
rm main.erl

lake build

./.lake/build/bin/ftorlisp ./examples/main.ftl \
  --stdlib ./Stdlib/stdlib.ftl \
  --module main \
  -o main.erl

erlc ./stdlib.erl ./main.erl

erl -pa ./ -eval "c:c(stdlib), c:c(main), main:main()" -noshell -s init stop 
```
