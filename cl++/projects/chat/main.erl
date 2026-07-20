-module(main).
-export([start/0]).











start() ->
    try
        Server = server:server(),
    throw({'__clx_return', accpet_loop(Server)})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.

accpet_loop(Server) ->
    try
        Response = gen_tcp:accept(Server),
    Client = clx_std:get_element(Response, 2),
    case clx_std:to_boolean(clx_std:get_element(Response, 1) == ok) of
    true -> 
        Worker = spawn(fun() ->
    try
        Db = db:db(),
        throw({'__clx_return', auth:auth(Client, Db)})
    catch
        throw:{'__clx_return', AnonymReturnValue} -> 
        AnonymReturnValue
        end
    end),
    gen_tcp:controlling_process(Client, Worker);
    _ ->
        ok
end,
    throw({'__clx_return', accpet_loop(Server)})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.