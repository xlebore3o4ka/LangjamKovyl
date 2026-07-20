-module(server).
-export([server/0]).

server() ->
    try
        Port = 8080,
    Options = [binary, {packet, 0}, {active, true}, {reuseaddr, true}],
    Listen_response = gen_tcp:listen(Port, Options),
    Socket = clx_std:get_element(Listen_response, 2),
    clx_std:print("Server start on port " ++ integer_to_list(Port)),
    throw({'__clx_return', Socket})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.