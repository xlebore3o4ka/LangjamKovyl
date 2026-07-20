(dec print       [String] String)
(dec println     [String] String)
(dec print_num   [Number] Number)
(dec println_num [Number] Number)

(dec lt  [Number Number] Bool)
(dec gt  [Number Number] Bool)
(dec lte [Number Number] Bool)
(dec gte [Number Number] Bool)
(dec neq [Number Number] Bool)

(dec ft_mod  [Number Number] Number)
(dec ft_abs  [Number] Number)
(dec ft_min  [Number Number] Number)
(dec ft_max  [Number Number] Number)
(dec ft_pow  [Number Number] Number)
(dec ft_sqrt [Number] Number)

(dec str_concat   [String String] String)
(dec str_len      [String] Number)
(dec str_upper    [String] String)
(dec str_lower    [String] String)
(dec str_trim     [String] String)
(dec str_contains [String String] Bool)

(dec number_to_string  [Number] String)
(dec string_to_number  [String] Number)
(dec bool_to_string [Bool] String)

(dec read_line [] String)

(dec str_eq [String String] Bool)
(dec str_split_once [String String] (List String))

(dec str_list_take [Number (List String)] (List String))
(dec str_list_remove [String (List String)] (List String))
(dec str_list_contains [String (List String)] Bool)