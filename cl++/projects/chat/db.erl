-module(db).
-export([db/0]).

db() ->
    try
        Options = [binary, {packet, 0}, {active, false}],
    Ip = "127.0.0.1",
    Port = 6379,
    Response = gen_tcp:connect(Ip, Port, Options),
    throw({'__clx_return', clx_std:get_element(Response, 2)})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.