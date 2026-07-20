-module(chat_room).
-export([chat_room/5]).







chat_room(Client,Db,My_nick,Target_nick,Chat_type) ->
    try
        clear_screen:clear_screen(Client),
    case clx_std:to_boolean(Chat_type == group) of
    true -> 
        load_group_history(Client, Db, Target_nick);
    _ ->
        ok
end,
    gen_tcp:send(Client, unicode:characters_to_binary("[ " ++ "Вы вошли в чат с " ++ Target_nick ++ " | " ++ "/exit - чтоб выйти с чата, /help - напомнить команды" ++ "| " ++ "Приятного общения!:)" ++ " ]\n")),
    throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick, Chat_type)})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

chat_receive_loop(Client,Db,My_nick,Target_nick,Chat_type) ->
    try
        receive
 {tcp, Client, Data} ->
        Text = clean_input:clean_input(Data),
        case clx_std:to_boolean(Text == "/exit") of
    true -> 
        throw({'__clx_return', menu:menu(Client, Db, My_nick)});
    _ ->
        ok
end,
        Parts = string:split(Text, " ", all),
        case clx_std:to_boolean((clx_std:get_element(Parts, 1) == "/add" andalso Chat_type == group)) of
    true -> 
        User_to_add = clx_std:get_element(Parts, 2),
    gen_tcp:send(Db, unicode:characters_to_binary("SADD group_members:" ++ Target_nick ++ " " ++ User_to_add ++ "\r\n")),
    gen_tcp:recv(Db, 0),
    gen_tcp:send(Db, unicode:characters_to_binary("SADD user_chats:" ++ User_to_add ++ " " ++ Target_nick ++ "\r\n")),
    gen_tcp:recv(Db, 0),
    gen_tcp:send(Client, "\e[1A\e[2K\r"),
    gen_tcp:send(Client, unicode:characters_to_binary("Вы добавили " ++ User_to_add ++ " в группу.\n")),
    throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick, Chat_type)});
    _ ->
        ok
end,
        case clx_std:to_boolean((clx_std:get_element(Parts, 1) == "/kick" andalso Chat_type == group)) of
    true -> 
        User_to_kick = clx_std:get_element(Parts, 2),
    gen_tcp:send(Db, unicode:characters_to_binary("SREM group_members:" ++ Target_nick ++ " " ++ User_to_kick ++ "\r\n")),
    gen_tcp:recv(Db, 0),
    gen_tcp:send(Db, unicode:characters_to_binary("SREM user_chats:" ++ User_to_kick ++ " " ++ Target_nick ++ "\r\n")),
    gen_tcp:recv(Db, 0),
    gen_tcp:send(Client, "\e[1A\e[2K\r"),
    gen_tcp:send(Client, unicode:characters_to_binary("Вы кикнули " ++ User_to_kick ++ " из группы.\n")),
    throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick, Chat_type)});
    _ ->
        ok
end,
        case clx_std:to_boolean(Text == "/help") of
    true -> 
        gen_tcp:send(Client, unicode:characters_to_binary("Доступные команды: /exit - выйти, /help - помощь\n")),
    throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick, Chat_type)});
    _ ->
        ok
end,
        case clx_std:to_boolean(Text == "") of
    true -> 
        throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick, Chat_type)});
    _ ->
        ok
end,
        Time_str = get_formatted_time(),
        Formatted_msg = "[" ++ Time_str ++ "] " ++ My_nick ++ ": " ++ Text,
        case clx_std:to_boolean(Chat_type == group) of
    true -> 
        broadcast_group_message(Db, Target_nick, My_nick, Formatted_msg);
    _ ->
        ok
end,
        gen_tcp:send(Client, unicode:characters_to_binary("\e[1A\e[2K\r" ++ Formatted_msg ++ "\n")),
        throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick, Chat_type)});
    {chat_msg, Incoming_text} ->
        gen_tcp:send(Client, unicode:characters_to_binary(Incoming_text ++ "\n")),
        throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick, Chat_type)});
    {tcp_closed, _client} ->
        throw({'__clx_return', exit(normal)})
end
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

broadcast_group_message(Db,Group_name,My_nick,Message) ->
    try
        Chat_key = "chat:group:" ++ Group_name,
    gen_tcp:send(Db, unicode:characters_to_binary("RPUSH " ++ Chat_key ++ " " ++ [34] ++ Message ++ [34] ++ "\r\n")),
    gen_tcp:recv(Db, 0),
    gen_tcp:send(Db, unicode:characters_to_binary("SMEMBERS group_members:" ++ Group_name ++ "\r\n")),
    Response = gen_tcp:recv(Db, 0),
    Lines = string:split(binary_to_list(clx_std:get_element(Response, 2)), "\r\n", all),
    lists:foreach(fun(Line) ->
    case clx_std:to_boolean(Line /= "") of
    true -> 
        Tr_char = try clx_std:get_element(Line, 1) of __TryRes -> {ok, __TryRes} catch _:__TryErr -> {error, __TryErr} end,
    case clx_std:to_boolean(clx_std:get_element(Tr_char, 1) == ok) of
    true -> 
        First_char = clx_std:get_element(Tr_char, 2),
    case clx_std:to_boolean((First_char /= 34 andalso First_char /= 42)) of
    true -> 
        Member_nick = Line,
    case clx_std:to_boolean(Member_nick /= My_nick) of
    true -> 
        Member_atom = list_to_atom(Member_nick),
    Member_pid = whereis(Member_atom),
    case clx_std:to_boolean(Member_pid /= undefined) of
    true -> 
        erlang:send(Member_pid, {chat_msg, Message});
    _ ->
        ok
end;
    _ ->
        ok
end;
    _ ->
        ok
end;
    _ ->
        ok
end;
    _ ->
        ok
end
end, Lines),
    throw({'__clx_return', ok})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

load_group_history(Client,Db,Group_name) ->
    try
        gen_tcp:send(Db, unicode:characters_to_binary("LRANGE " ++ "chat:group:" ++ Group_name ++ " 0 -1\r\n")),
    Response = recv_all_redis(Db, ""),
    Lines = string:split(Response, "\r\n", all),
    lists:foreach(fun(Line) ->
    case clx_std:to_boolean(Line /= "") of
    true -> 
        Decoded = unicode:characters_to_list(list_to_binary(Line)),
    Clean_line = fun() ->
    try
        Tr_tag = try clx_std:get_element(Decoded, 1) of __TryRes -> {ok, __TryRes} catch _:__TryErr -> {error, __TryErr} end,
        case clx_std:to_boolean(clx_std:get_element(Tr_tag, 1) == ok) of
    true -> 
        Tag = clx_std:get_element(Tr_tag, 2),
    case clx_std:to_boolean(Tag == incomplete) of
    true -> 
        throw({'__clx_return', clx_std:get_element(Decoded, 2)});
    _ ->
        ok
end;
    _ ->
        ok
end,
        throw({'__clx_return', Decoded})
    catch
        throw:{'__clx_return', AnonymReturnValue} -> 
        AnonymReturnValue
        end
    end(),
    Tr_char = try clx_std:get_element(Clean_line, 1) of __TryRes -> {ok, __TryRes} catch _:__TryErr -> {error, __TryErr} end,
    case clx_std:to_boolean(clx_std:get_element(Tr_char, 1) == ok) of
    true -> 
        First_char = clx_std:get_element(Tr_char, 2),
    case clx_std:to_boolean((First_char /= 36 andalso First_char /= 42)) of
    true -> 
        gen_tcp:send(Client, unicode:characters_to_binary(Clean_line ++ "\n"));
    _ ->
        ok
end;
    _ ->
        ok
end;
    _ ->
        ok
end
end, Lines)
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

load_chat_history(Client,Db,My_nick,Target_nick) ->
    try
        Chat_key = get_private_key(My_nick, Target_nick),
    Lrange = "LRANGE " ++ Chat_key ++ " 0 -1\r\n",
    gen_tcp:send(Db, unicode:characters_to_binary(Lrange)),
    Response = recv_all_redis(Db, ""),
    Lines = string:split(Response, "\r\n", all),
    lists:foreach(fun(Line) ->
    case clx_std:to_boolean(Line /= "") of
    true -> 
        Decoded = unicode:characters_to_list(list_to_binary(Line)),
    Clean_line = fun() ->
    try
        Tr_tag = try clx_std:get_element(Decoded, 1) of __TryRes -> {ok, __TryRes} catch _:__TryErr -> {error, __TryErr} end,
        case clx_std:to_boolean(clx_std:get_element(Tr_tag, 1) == ok) of
    true -> 
        Tag = clx_std:get_element(Tr_tag, 2),
    case clx_std:to_boolean(Tag == incomplete) of
    true -> 
        throw({'__clx_return', clx_std:get_element(Decoded, 2)});
    _ ->
        ok
end;
    _ ->
        ok
end,
        throw({'__clx_return', Decoded})
    catch
        throw:{'__clx_return', AnonymReturnValue} -> 
        AnonymReturnValue
        end
    end(),
    Tr_char = try clx_std:get_element(Clean_line, 1) of __TryRes -> {ok, __TryRes} catch _:__TryErr -> {error, __TryErr} end,
    case clx_std:to_boolean(clx_std:get_element(Tr_char, 1) == ok) of
    true -> 
        First_char = clx_std:get_element(Tr_char, 2),
    case clx_std:to_boolean((First_char /= 36 andalso First_char /= 42)) of
    true -> 
        gen_tcp:send(Client, unicode:characters_to_binary(Clean_line ++ "\n"));
    _ ->
        ok
end;
    _ ->
        ok
end;
    _ ->
        ok
end
end, Lines)
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

recv_all_redis(Db,Current_str) ->
    try
        Lines = string:split(Current_str, "\r\n", all),
    Current_lines_count = erlang:length(Lines),
    case clx_std:to_boolean(Current_lines_count < 2) of
    true -> 
        Recv_res = gen_tcp:recv(Db, 0),
    case clx_std:to_boolean(clx_std:get_element(Recv_res, 1) == ok) of
    true -> 
        Chunk = binary_to_list(clx_std:get_element(Recv_res, 2)),
    throw({'__clx_return', recv_all_redis(Db, Current_str ++ Chunk)});
    _ ->
        ok
end;
    _ ->
        ok
end
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

get_formatted_time() ->
    try
        Seconds = erlang:system_time(second),
    Datetime = calendar:system_time_to_universal_time(Seconds, second),
    Time_tuple = clx_std:get_element(Datetime, 2),
    Hours = integer_to_list(clx_std:get_element(Time_tuple, 1)),
    Raw_minutes = integer_to_list(clx_std:get_element(Time_tuple, 2)),
    Minutes = fun() ->
    try
        case clx_std:to_boolean(clx_std:get_element(Time_tuple, 2) < 10) of
    true -> 
        throw({'__clx_return', "0" ++ Raw_minutes});
    _ ->
        ok
end
    catch
        throw:{'__clx_return', AnonymReturnValue} -> 
        AnonymReturnValue
        end
    end(),
    throw({'__clx_return', Hours ++ ":" ++ Minutes})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

get_private_key(User_a,User_b) ->
    try
        case clx_std:to_boolean(User_a < User_b) of
    true -> 
        throw({'__clx_return', "chat:private:" ++ User_a ++ ":" ++ User_b});
    _ ->
        ok
end
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

save_messages_to_db(Db,From_user,To_user,Formatted_msg) ->
    try
        Chat_key = get_private_key(From_user, To_user),
    Rpush = "RPUSH " ++ Chat_key ++ " " ++ [34] ++ Formatted_msg ++ [34] ++ "\r\n",
    gen_tcp:send(Db, unicode:characters_to_binary(Rpush)),
    gen_tcp:recv(Db, 0),
    Sadd_from = "SADD user_chats:" ++ From_user ++ " " ++ To_user ++ "\r\n",
    gen_tcp:send(Db, unicode:characters_to_binary(Sadd_from)),
    gen_tcp:recv(Db, 0),
    Sadd_to = "SADD user_chats:" ++ To_user ++ " " ++ From_user ++ "\r\n",
    gen_tcp:send(Db, unicode:characters_to_binary(Sadd_to)),
    gen_tcp:recv(Db, 0),
    throw({'__clx_return', ok})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.