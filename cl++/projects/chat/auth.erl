-module(auth).
-export([auth/2]).







auth(Client,Db) ->
    try
        clear_screen:clear_screen(Client),
    gen_tcp:send(Client, unicode:characters_to_binary("Приветствуем в нашем NetChat!\n1 - ввойти в аккаунт\n2 - создать аккаунт\n:")),
    Command = receive_input:receive_input(),
    clear_screen:clear_screen(Client),
    case clx_std:to_boolean(Command == "1") of
    true -> 
        throw({'__clx_return', handle_login(Client, Db)});
    _ ->
        case clx_std:to_boolean(Command == "2") of
    true -> 
        throw({'__clx_return', handle_reg(Client, Db)});
    _ ->
        clear_screen:clear_screen(Client),
    gen_tcp:send(Client, unicode:characters_to_binary("\nВведена неправильная команда. Попробуйте ещё раз!")),
    timer:sleep(2000),
    throw({'__clx_return', auth(Client, Db)})
end
end
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

handle_login(Client,Db) ->
    try
        Data = form(Client),
    Get = "GET account:" ++ clx_std:get_element(Data, 1) ++ "\r\n",
    gen_tcp:send(Db, unicode:characters_to_binary(Get)),
    Response = gen_tcp:recv(Db, 0),
    Answer = binary_to_list(clx_std:get_element(Response, 2)),
    case clx_std:to_boolean(Answer == "$-1\r\n") of
    true -> 
        gen_tcp:send(Client, unicode:characters_to_binary("Ошибка: аккаунт не найден!")),
    timer:sleep(2000),
    throw({'__clx_return', auth(Client, Db)});
    _ ->
        Real_password = string:split(Answer, "\r\n", all),
    clear_screen:clear_screen(Client),
    case clx_std:to_boolean(clx_std:get_element(Data, 2) == clx_std:get_element(Real_password, 2)) of
    true -> 
        gen_tcp:send(Client, unicode:characters_to_binary("\nВы успешно вошли!")),
    timer:sleep(2000),
    register(list_to_atom(clx_std:get_element(Data, 1)), self()),
    throw({'__clx_return', menu:menu(Client, Db, clx_std:get_element(Data, 1))});
    _ ->
        gen_tcp:send(Client, unicode:characters_to_binary("\nПароль не верный!")),
    timer:sleep(2000),
    throw({'__clx_return', auth(Client, Db)})
end
end
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

handle_reg(Client,Db) ->
    try
        Data = form(Client),
    Get = "GET account:" ++ clx_std:get_element(Data, 1) ++ "\r\n",
    gen_tcp:send(Db, unicode:characters_to_binary(Get)),
    Response = gen_tcp:recv(Db, 0),
    Answer = binary_to_list(clx_std:get_element(Response, 2)),
    case clx_std:to_boolean(Answer == "$-1\r\n") of
    true -> 
        Set = "SET account:" ++ clx_std:get_element(Data, 1) ++ " " ++ clx_std:get_element(Data, 2) ++ "\r\n",
    gen_tcp:send(Db, unicode:characters_to_binary(Set)),
    gen_tcp:recv(Db, 0),
    gen_tcp:send(Client, unicode:characters_to_binary("\nУспешная регистрация!\n")),
    timer:sleep(2000),
    register(list_to_atom(clx_std:get_element(Data, 1)), self()),
    throw({'__clx_return', menu:menu(Client, Db, clx_std:get_element(Data, 1))});
    _ ->
        gen_tcp:send(Client, unicode:characters_to_binary("\nОшибка, ник уже занят!\n")),
    timer:sleep(2000),
    clear_screen:clear_screen(Client),
    throw({'__clx_return', auth(Client, Db)})
end
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

form(Client) ->
    try
        gen_tcp:send(Client, unicode:characters_to_binary("Введите имя пользователя: ")),
    Username = receive_input:receive_input(),
    gen_tcp:send(Client, unicode:characters_to_binary("\nВведите ваш пароль: ")),
    Password = receive_input:receive_input(),
    throw({'__clx_return', [Username, Password]})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.