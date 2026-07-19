-module(receive_input).
-export([receive_input/0]).



receive_input() ->
    try
        throw({'__clx_return', fun() ->
    try
        receive
 {tcp, _client, Data} ->
        throw({'__clx_return', clean_input:clean_input(Data)});
    {tcp_closed, _client} ->
        throw({'__clx_return', exit(normal)})
end
    catch
        throw:{'__clx_return', AnonymReturnValue} -> 
        AnonymReturnValue
        end
    end()})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.