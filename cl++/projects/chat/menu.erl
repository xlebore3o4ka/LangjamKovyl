-module(menu).
-export([menu/3]).







menu(Client,Db,Nickname) ->
    try
        clear_screen:clear_screen(Client),
    gen_tcp:send(Client, unicode:characters_to_binary("\n--- Главное Меню ---\n")),
    Get_chats = "SMEMBERS user_chats:" ++ Nickname ++ "\r\n",
    gen_tcp:send(Db, unicode:characters_to_binary(Get_chats)),
    Chats = gen_tcp:recv(Db, 0),
    Chat_lines = string:split(binary_to_list(clx_std:get_element(Chats, 2)), "\r\n", all),
    gen_tcp:send(Client, unicode:characters_to_binary("Ваши чаты:\n")),
    display_chats(Client, Chat_lines),
    gen_tcp:send(Client, unicode:characters_to_binary("Введите имя пользоватея или чата\n")),
    gen_tcp:send(Client, unicode:characters_to_binary(":")),
    Target_nickname = receive_input:receive_input(),
    case clx_std:to_boolean(Target_nickname == "") of
    true -> 
        throw({'__clx_return', menu(Client, Db, Nickname)});
    _ ->
        ok
end,
    Get = "GET account:" ++ Target_nickname ++ "\r\n",
    gen_tcp:send(Db, Get),
    Response = string:split(binary_to_list(clx_std:get_element(gen_tcp:recv(Db, 0), 2)), "\r\n", all),
    case clx_std:to_boolean(clx_std:get_element(Response, 1) == "$-1") of
    true -> 
        gen_tcp:send(Client, unicode:characters_to_binary("\nОшибка: Такого пользователя не существует!")),
    timer:sleep(2000),
    throw({'__clx_return', menu(Client, Db, Nickname)});
    _ ->
        throw({'__clx_return', chat_room:chat_room(Client, Db, Nickname, Target_nickname)})
end
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

display_chats(Client,Chat_lines) ->
    try
        lists:foreach(fun(Line) ->
    case clx_std:to_boolean(Line /= "") of
    true -> 
        Tr_char = try clx_std:get_element(Line, 1) of __TryRes -> {ok, __TryRes} catch _:__TryErr -> {error, __TryErr} end,
    case clx_std:to_boolean(clx_std:get_element(Tr_char, 1) == ok) of
    true -> 
        First_char = clx_std:get_element(Tr_char, 2),
    case clx_std:to_boolean((First_char /= 36 andalso First_char /= 42)) of
    true -> 
        gen_tcp:send(Client, unicode:characters_to_binary(" - " ++ Line ++ "\n"));
    _ ->
        ok
end;
    _ ->
        ok
end;
    _ ->
        ok
end
end, Chat_lines)
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.