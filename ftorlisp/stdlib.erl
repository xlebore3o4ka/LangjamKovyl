-module(stdlib).

-export([
    %% IO
    print/1, println/1, print_num/1, println_num/1,

    %% Сравнения
    lt/2, gt/2, lte/2, gte/2, neq/2,

    %% Математика
    ft_mod/2, ft_abs/1, ft_min/2, ft_max/2, ft_pow/2, ft_sqrt/1,

    %% Строки
    str_concat/2, str_len/1, str_upper/1, str_lower/1, str_trim/1,
    str_split/2, str_contains/2,

    %% Конвертация
    number_to_string/1, string_to_number/1, bool_to_string/1,

    %% Списки (нативно полиморфны в Erlang, но
    %% на стороне Ftorlisp это должны быть спецформы, см. ниже)
    ft_list_length/1, ft_list_is_empty/1, ft_list_reverse/1,
    ft_list_append/2, ft_list_nth/2,

    str_eq/2, read_line/0, str_split_once/2,
    str_list_take/2, str_list_remove/2, str_list_contains/2
]).

%% ==================== IO ====================
%% Возвращаем то же значение, что напечатали — юнита у нас нет.

print(Str) ->
    io:format("~ts", [Str]),
    Str.

println(Str) ->
    io:format("~ts~n", [Str]),
    Str.

print_num(Num) ->
    io:format("~p", [Num]),
    Num.

println_num(Num) ->
    io:format("~p~n", [Num]),
    Num.

%% ==================== Сравнения ====================
%% Bool у нас компилируется в атомы true/false — как в Erlang.

lt(A, B)  -> A < B.
gt(A, B)  -> A > B.
lte(A, B) -> A =< B.
gte(A, B) -> A >= B.
neq(A, B) -> A /= B.

%% ==================== Математика ====================
%% Числа внутри — Float, поэтому mod считаем через rem с округлением.

ft_mod(A, B) ->
    (trunc(A) rem trunc(B)) * 1.0.

ft_abs(A) -> erlang:abs(A).
ft_min(A, B) -> erlang:min(A, B).
ft_max(A, B) -> erlang:max(A, B).
ft_pow(Base, Exp) -> math:pow(Base, Exp).
ft_sqrt(A) -> math:sqrt(A).

%% ==================== Строки ====================
%% String компилируется в обычный Erlang-список символов.

str_concat(A, B) -> A ++ B.
str_len(A) -> length(A) * 1.0.
str_upper(A) -> string:uppercase(A).
str_lower(A) -> string:lowercase(A).
str_trim(A) -> string:trim(A).

str_split(Str, Sep) ->
    string:split(Str, Sep, all).

str_contains(Str, Sub) ->
    case string:find(Str, Sub) of
        nomatch -> false;
        _ -> true
    end.

%% ==================== Конвертация ====================

number_to_string(Num) ->
    lists:flatten(io_lib:format("~p", [trunc(Num)])).

string_to_number(Str) ->
    try list_to_float(Str)
    catch _:_ -> float(list_to_float(Str))
    end.

bool_to_string(true)  -> "true";
bool_to_string(false) -> "false".

%% ==================== Списки ====================
%% Работают полиморфно вне зависимости от типа элемента —
%% на стороне Ftorlisp это должно быть спецформами (см. ниже).

ft_list_length(L) -> length(L) * 1.0.

ft_list_is_empty([]) -> true;
ft_list_is_empty(_)  -> false.

ft_list_reverse(L) -> lists:reverse(L).

ft_list_append(A, B) -> A ++ B.

ft_list_nth(N, L) -> lists:nth(trunc(N) + 1, L). % 0-индексация -> 1-индексация


str_eq(A, B) -> 
    A =:= B.

read_line() ->
    %% Считываем строку из консоли и обрезаем перенос строки
    case io:get_line("") of
        eof -> "";
        Str -> string:trim(Str, trailing, "\n")
    end.

str_split_once(Str, Sep) ->
    %% Разделяем строку на 2 части по первому вхождению (команда + аргумент)
    string:split(Str, Sep, leading).

%% ==================== Списки строк ====================

str_list_take(N, L) ->
    %% N - float, т.к. числа во Ftorlisp это Float
    lists:sublist(L, trunc(N)).

str_list_remove(Item, L) ->
    %% Удаляем первый встречный элемент
    lists:delete(Item, L).

str_list_contains(Item, L) ->
    lists:member(Item, L).