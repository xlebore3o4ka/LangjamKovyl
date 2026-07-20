
(data Command
  (cmd_add String)
  (cmd_leave String)
  (cmd_send String)
  (cmd_hist Number)
  (cmd_quit)
  (cmd_unknown))

(dec not [Bool] Bool)
(def not [a]
	(= a false))

(dec str_list_is_empty [(List String)] Bool)
(def str_list_is_empty [msgs]
  (= msgs 'String[]))

(dec parse_cmd_nonempty [(List String)] Command)
(dec parse_cmd [(List String)] Command)
(def parse_cmd [tokens]
  (if (str_list_is_empty tokens)
      (cmd_unknown)
      (parse_cmd_nonempty tokens)))

(def parse_cmd_nonempty [tokens]
  (let cmd (first tokens))
  (let rest_toks (rest tokens))
  (let has_arg (not (str_list_is_empty rest_toks)))
  
  (if (str_eq cmd "quit") (cmd_quit)
  (if (str_eq cmd "add")
      (if has_arg (cmd_add (first rest_toks)) (cmd_unknown))
  (if (str_eq cmd "leave")
      (if has_arg (cmd_leave (first rest_toks)) (cmd_unknown))
  (if (str_eq cmd "send")
      (if has_arg (cmd_send (first rest_toks)) (cmd_unknown))
  (if (str_eq cmd "hist")
      (if has_arg (cmd_hist (string_to_number (first rest_toks))) (cmd_hist 5))
  (cmd_unknown)))))))

(dec chat_loop [(List String) (List String)] Bool)

(dec chat_loop_with_msg [String (List String) (List String)] Bool)
(def chat_loop_with_msg [msg users msgs]
  (let dummy (println msg))
  (chat_loop users msgs))

(dec print_history_step [(List String)] Bool)

(dec print_history [(List String)] Bool)
(def print_history [msgs]
  (if (str_list_is_empty msgs)
      true
      (print_history_step msgs)))

(def print_history_step [msgs]
  (let dummy (println (first msgs)))
  (print_history (rest msgs)))

(dec handle_add [String (List String) (List String)] Bool)
(def handle_add [name users msgs]
  (if (str_list_contains name users)
      (chat_loop_with_msg "[-] Ошибка: Пользователь уже в чате!" users msgs)
      (chat_loop_with_msg (str_concat "[+] Вошел в чат: " name) (cons name users) msgs)))

(dec handle_leave [String (List String) (List String)] Bool)
(def handle_leave [name users msgs]
  (if (str_list_contains name users)
      (chat_loop_with_msg (str_concat "[-] Покинул чат: " name) (str_list_remove name users) msgs)
      (chat_loop_with_msg "[-] Ошибка: Такого пользователя нет." users msgs)))

(dec handle_send [String (List String) (List String)] Bool)
(def handle_send [text users msgs]
  (let msg_full (str_concat "[Всем]: " text))
  (let info "Сообщение доставлено участникам: ")
  (let dummy (println info))
  (chat_loop users (cons msg_full msgs)))

(dec handle_hist [Number (List String) (List String)] Bool)
(def handle_hist [n users msgs]
  (let last_n (str_list_take n msgs))
  (let dummy1 (println "--- История сообщений ---"))
  (let dummy2 (print_history last_n))
  (chat_loop users msgs))

(dec handle_quit [] Bool)
(def handle_quit []
  (let dummy (println "Выход из чата. Пока!"))
  true)

(dec handle_unknown [(List String) (List String)] Bool)
(def handle_unknown [users msgs]
  (let dummy (println "Неизвестная команда. Доступно: add <имя>, leave <имя>, send <текст>, hist <N.0 (обязательно с точкой и нулём!)>, quit"))
  (chat_loop users msgs))

(def chat_loop [users msgs]
  (let dummy_prompt (print "> "))
  (let input (read_line))
  (let tokens (str_split_once input " "))
  (let cmd (parse_cmd tokens))

  (match cmd
    [(cmd_add name)  (handle_add name users msgs)]
    [(cmd_leave name)(handle_leave name users msgs)]
    [(cmd_send text) (handle_send text users msgs)]
    [(cmd_hist n)    (handle_hist n users msgs)]
    [(cmd_quit)      (handle_quit)]
    [(cmd_unknown)   (handle_unknown users msgs)]))

(println "Доступно: add <имя>, leave <имя>, send <текст>, hist <N.0 (обязательно с точкой и нулём!)>, quit")
(chat_loop 'String[] 'String[])