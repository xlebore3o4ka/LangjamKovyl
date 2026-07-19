-module(chat_room).
-export([chat_room/4]).







chat_room(Client,Db,My_nick,Target_nick) ->
    try
        clear_screen:clear_screen(Client),
    gen_tcp:send(Client, unicode:characters_to_binary("\n--- Чат с " ++ Target_nick ++ " ---\n(Введи /exit для выхода)\n\n")),
    throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick)})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

chat_receive_loop(Client,Db,My_nick,Target_nick) ->
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
        case clx_std:to_boolean(Text == "") of
    true -> 
        throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick)});
    _ ->
        ok
end,
        Time_str = get_formatted_time(),
        Formatted_msg = "[" ++ Time_str ++ "] " ++ My_nick ++ ": " ++ Text,
        save_messages_to_db(Db, My_nick, Target_nick, Formatted_msg),
        Target_atom = list_to_atom(Target_nick),
        Target_pid = whereis(Target_atom),
        case clx_std:to_boolean(Target_pid /= undefined) of
    true -> 
        erlang:send(Target_pid, {chat_msg, Formatted_msg});
    _ ->
        ok
end,
        gen_tcp:send(Client, unicode:characters_to_binary("\e[1A\e[2K\r" ++ Formatted_msg ++ "\n")),
        throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick)});
    {chat_msg, Incoming_text} ->
        gen_tcp:send(Client, unicode:characters_to_binary(Incoming_text ++ "\n")),
        throw({'__clx_return', chat_receive_loop(Client, Db, My_nick, Target_nick)});
    {tcp_closed, _client} ->
        throw({'__clx_return', exit(normal)})
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
        throw({'__clx_return', Raw_minutes})
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
        throw({'__clx_return', "chat:private:" ++ User_b ++ ":" ++ User_a})
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