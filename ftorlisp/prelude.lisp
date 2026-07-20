(dec and [Bool Bool] Bool)
(def and [a b]
  (= a b true))

(dec not [Bool] Bool)
(def not [a]
	(= a false))

(dec or [Bool Bool] Bool)
(def or [a b]
	(not (= a b false)))

(dec xor [Bool Bool] Bool)
(def xor [a b]
	(not (= a b)))