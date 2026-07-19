-module(clean_input).
-export([clean_input/1]).

clean_input(Input) ->
    try
        Bytes_size = byte_size(Input) - 1,
    case clx_std:to_boolean(Bytes_size =< 0) of
    true -> 
        throw({'__clx_return', ""});
    _ ->
        ok
end,
    Cleaned_data = binary_part(Input, 0, Bytes_size),
    Parsed_data = unicode:characters_to_list(Cleaned_data),
    Trimmed_data = string:trim(Parsed_data),
    throw({'__clx_return', Trimmed_data})
    catch
        throw:{'__clx_return', ReturnValue} -> 
        ReturnValue
    end.