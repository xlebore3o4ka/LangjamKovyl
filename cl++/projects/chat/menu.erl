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
    gen_tcp:send(Client, unicode:characters_to_binary("Ваши чаты и группы:\n")),
    display_chats(Client, Chat_lines),
    gen_tcp:send(Client, unicode:characters_to_binary("Введите имя пользоватея или группы\n")),
    gen_tcp:send(Client, unicode:characters_to_binary(":")),
    Input = receive_input:receive_input(),
    case clx_std:to_boolean(Input == "") of
    true -> 
        throw({'__clx_return', menu(Client, Db, Nickname)});
    _ ->
        ok
end,
    Parts = string:split(Input, " ", all),
    case clx_std:to_boolean(clx_std:get_element(Parts, 1) == "/group") of
    true -> 
        Group_name = clx_std:get_element(Parts, 2),
    throw({'__clx_return', create_group(Client, Db, Nickname, Group_name)});
    _ ->
        ok
end,
    throw({'__clx_return', enter_chat_or_group(Client, Db, Nickname, Input)})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

create_group(Client,Db,Nickname,Group_name) ->
    try
        case clx_std:to_boolean(Group_name == "") of
    true -> 
        gen_tcp:send(Client, unicode:characters_to_binary("\nОшибка: Укажите имя группы.\n")),
    timer:sleep(2000),
    throw({'__clx_return', menu(Client, Db, Nickname)});
    _ ->
        ok
end,
    gen_tcp:send(Db, unicode:characters_to_binary("EXISTS account:" ++ Group_name ++ "\r\n")),
    User_exists = binary_to_list(clx_std:get_element(gen_tcp:recv(Db, 0), 2)),
    gen_tcp:send(Db, unicode:characters_to_binary("EXISTS group:" ++ Group_name ++ "\r\n")),
    Group_exists = binary_to_list(clx_std:get_element(gen_tcp:recv(Db, 0), 2)),
    case clx_std:to_boolean((User_exists == ":1\r\n" orelse Group_exists == ":1\r\n")) of
    true -> 
        gen_tcp:send(Client, unicode:characters_to_binary("\nОшибка: Имя уже занято.\n")),
    timer:sleep(2000),
    throw({'__clx_return', menu(Client, Db, Nickname)});
    _ ->
        ok
end,
    gen_tcp:send(Db, unicode:characters_to_binary("SET group:" ++ Group_name ++ " 1\r\n")),
    gen_tcp:recv(Db, 0),
    gen_tcp:send(Db, unicode:characters_to_binary("SADD group_members:" ++ Group_name ++ " " ++ Nickname ++ "\r\n")),
    gen_tcp:recv(Db, 0),
    gen_tcp:send(Db, unicode:characters_to_binary("SADD user_chats:" ++ Nickname ++ " " ++ Group_name ++ "\r\n")),
    gen_tcp:recv(Db, 0),
    throw({'__clx_return', chat_room:chat_room(Client, Db, Nickname, Group_name, group)})
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

enter_chat_or_group(Client,Db,Nickname,Target_name) ->
    try
        gen_tcp:send(Db, unicode:characters_to_binary("EXISTS account:" ++ Target_name ++ "\r\n")),
    User = binary_to_list(clx_std:get_element(gen_tcp:recv(Db, 0), 2)),
    case clx_std:to_boolean(User == ":1\r\n") of
    true -> 
        throw({'__clx_return', chat_room:chat_room(Client, Db, Nickname, Target_name, private)});
    _ ->
        ok
end,
    gen_tcp:send(Db, unicode:characters_to_binary("EXISTS group:" ++ Target_name ++ "\r\n")),
    Group = binary_to_list(clx_std:get_element(gen_tcp:recv(Db, 0), 2)),
    case clx_std:to_boolean(Group == ":1\r\n") of
    true -> 
        Check_member = "SISMEMBER group_members:" ++ Target_name ++ " " ++ Nickname ++ "\r\n",
    gen_tcp:send(Db, unicode:characters_to_binary(Check_member)),
    Is_member = binary_to_list(clx_std:get_element(gen_tcp:recv(Db, 0), 2)),
    case clx_std:to_boolean(Is_member == ":1\r\n") of
    true -> 
        throw({'__clx_return', chat_room:chat_room(Client, Db, Nickname, Target_name, group)});
    _ ->
        ok
end;
    _ ->
        ok
end,
    gen_tcp:send(Client, unicode:characters_to_binary("\nОшибка: Такого пользователя нет!")),
    timer:sleep(2000),
    throw({'__clx_return', menu(Client, Db, Nickname)})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.