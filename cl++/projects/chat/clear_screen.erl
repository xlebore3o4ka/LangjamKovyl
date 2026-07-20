-module(clear_screen).
-export([clear_screen/1]).

clear_screen(Client) ->
    try
        clx_std:print("User console cleared!"),
    gen_tcp:send(Client, "\e[r\e[2J\e[H")
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.