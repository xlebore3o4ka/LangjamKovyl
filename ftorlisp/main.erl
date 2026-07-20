-module(main).
-export([not_/1, str_list_is_empty/1, parse_cmd/1, parse_cmd_nonempty/1, chat_loop_with_msg/3, print_history/1, print_history_step/1, handle_add/3, handle_leave/3, handle_send/3, handle_hist/3, handle_quit/0, handle_unknown/2, chat_loop/2, main/0]).

%% data Command: cmd_add(String), cmd_leave(String), cmd_send(String), cmd_hist(Number), cmd_quit, cmd_unknown

not_(A) ->
    ((A =:= false)).

str_list_is_empty(Msgs) ->
    ((Msgs =:= [])).

parse_cmd(Tokens) ->
    (case str_list_is_empty(Tokens) of
        true -> {cmd_unknown};
        false -> parse_cmd_nonempty(Tokens)
    end).

parse_cmd_nonempty(Tokens) ->
    Cmd = hd(Tokens),
    Rest_toks = tl(Tokens),
    Has_arg = not_(str_list_is_empty(Rest_toks)),
    (case stdlib:str_eq(Cmd, "quit") of
        true -> {cmd_quit};
        false -> (case stdlib:str_eq(Cmd, "add") of
            true -> (case Has_arg of
                true -> {cmd_add, hd(Rest_toks)};
                false -> {cmd_unknown}
            end);
            false -> (case stdlib:str_eq(Cmd, "leave") of
                true -> (case Has_arg of
                    true -> {cmd_leave, hd(Rest_toks)};
                    false -> {cmd_unknown}
                end);
                false -> (case stdlib:str_eq(Cmd, "send") of
                    true -> (case Has_arg of
                        true -> {cmd_send, hd(Rest_toks)};
                        false -> {cmd_unknown}
                    end);
                    false -> (case stdlib:str_eq(Cmd, "hist") of
                        true -> (case Has_arg of
                            true -> {cmd_hist, stdlib:string_to_number(hd(Rest_toks))};
                            false -> {cmd_hist, 5.000000}
                        end);
                        false -> {cmd_unknown}
                    end)
                end)
            end)
        end)
    end).

chat_loop_with_msg(Msg, Users, Msgs) ->
    Dummy = stdlib:println(Msg),
    chat_loop(Users, Msgs).

print_history(Msgs) ->
    (case str_list_is_empty(Msgs) of
        true -> true;
        false -> print_history_step(Msgs)
    end).

print_history_step(Msgs) ->
    Dummy = stdlib:println(hd(Msgs)),
    print_history(tl(Msgs)).

handle_add(Name, Users, Msgs) ->
    (case stdlib:str_list_contains(Name, Users) of
        true -> chat_loop_with_msg("[-] Ошибка: Пользователь уже в чате!", Users, Msgs);
        false -> chat_loop_with_msg(stdlib:str_concat("[+] Вошел в чат: ", Name), [Name | Users], Msgs)
    end).

handle_leave(Name, Users, Msgs) ->
    (case stdlib:str_list_contains(Name, Users) of
        true -> chat_loop_with_msg(stdlib:str_concat("[-] Покинул чат: ", Name), stdlib:str_list_remove(Name, Users), Msgs);
        false -> chat_loop_with_msg("[-] Ошибка: Такого пользователя нет.", Users, Msgs)
    end).

handle_send(Text, Users, Msgs) ->
    Msg_full = stdlib:str_concat("[Всем]: ", Text),
    Info = "Сообщение доставлено участникам: ",
    Dummy = stdlib:println(Info),
    chat_loop(Users, [Msg_full | Msgs]).

handle_hist(N, Users, Msgs) ->
    Last_n = stdlib:str_list_take(N, Msgs),
    Dummy1 = stdlib:println("--- История сообщений ---"),
    Dummy2 = print_history(Last_n),
    chat_loop(Users, Msgs).

handle_quit() ->
    Dummy = stdlib:println("Выход из чата. Пока!"),
    true.

handle_unknown(Users, Msgs) ->
    Dummy = stdlib:println("Неизвестная команда. Доступно: add <имя>, leave <имя>, send <текст>, hist <N.0 (обязательно с точкой и нулём!)>, quit"),
    chat_loop(Users, Msgs).

chat_loop(Users, Msgs) ->
    Dummy_prompt = stdlib:print("> "),
    Input = stdlib:read_line(),
    Tokens = stdlib:str_split_once(Input, " "),
    Cmd = parse_cmd(Tokens),
    (case Cmd of
        {cmd_add, Name} -> handle_add(Name, Users, Msgs);
        {cmd_leave, Name} -> handle_leave(Name, Users, Msgs);
        {cmd_send, Text} -> handle_send(Text, Users, Msgs);
        {cmd_hist, N} -> handle_hist(N, Users, Msgs);
        {cmd_quit} -> handle_quit();
        {cmd_unknown} -> handle_unknown(Users, Msgs)
    end).

main() ->
    stdlib:println("Доступно: add <имя>, leave <имя>, send <текст>, hist <N.0 (обязательно с точкой и нулём!)>, quit"),
    chat_loop([], []).
