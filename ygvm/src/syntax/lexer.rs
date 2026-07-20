use std::str::FromStr;
use thiserror::Error;

pub struct Lexer {
    file: String,
    input: String,
    pos: usize,
    line: u32,
    column: u32,
}

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("[{0} / {1}, {2}] Unexpected character: {3}")]
    UnexpectedSymbol(String, u32, u32, char),

    #[error("[{0} / {1}, {2}] Number format error")]
    NumberFormatError(String, u32, u32),

    #[error("[{0} / {1}, {2}] String format error")]
    StringFormatError(String, u32, u32),
}

#[derive(Debug)]
pub struct Token {
    pub data: TokenData,
    pub debug: TokenDebug
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenData {
    IdModule,

    IdClass,
    IdObject,
    IdExtends,
    IdAnd,

    IdFun,
    IdDef,

    IdUse,
    IdAs,

    IdBlock,
    IdIf,
    IdElse,
    IdWhile,
    IdDo,
    IdFor,
    IdIn,
    IdTry,
    IdCatch,

    IdNew,
    IdLet,
    IdReturn,
    IdThrow,

    IdEnd,

    Ident(String),

    Boolean(bool),
    Integer(i64),
    Double(f64),
    String(String),

    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    Semicolon,      // ;
    Colon,          // :
    ColonColon,     // ::
    Dot,            // .
    Comma,          // ,

    Assign,         // =
    AssignPlus,     // =+
    AssignMinus,    // =-
    AssignStar,     // =*
    AssignSlash,    // =/
    AssignPercent,  // =%
    AssignAnd,      // =&
    AssignOr,       // =|
    AssignXor,      // =^
    AssignNot,      // =!
    AssignLt,       // =<
    AssignGt,       // =>
    AssignLe,       // =<=
    AssignGe,       // =>=

    Plus,           // +
    Increment,      // ++
    Minus,          // -
    Decrement,      // --
    Star,           // *
    StarStar,       // **
    Slash,          // /
    SlashSlash,     // //
    Percent,        // %

    And,            // &
    AndAnd,         // &&
    Or,             // |
    OrOr,           // ||
    Xor,            // ^
    XorXor,         // ^^
    Not,            // !

    Eq,             // ==
    Neq,            // !=
    Lt,             // <
    Gt,             // >
    Le,             // <=
    Ge,             // >=

    NewLine
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TokenDebug {
    pub line: u32,
    pub column: u32
}

impl Lexer {
    #[inline(always)]
    pub fn new(file: String, input: String) -> Self {
        Self { file, input, pos: 0, line: 0, column: 0 }
    }

    pub fn file(&self) -> &String {
        &self.file
    }

    pub fn source(&self) -> &String {
        &self.input
    }

    pub fn next(&mut self) -> Result<Option<Token>, LexerError> {
        self.skip_ws();
        if let Some(char) = self.input.chars().nth(self.pos) {
            self.inc_pos();
            match char {
                '('  => self.ok_token(TokenData::LParen),
                ')'  => self.ok_token(TokenData::RParen),
                '{'  => self.ok_token(TokenData::LBrace),
                '}'  => self.ok_token(TokenData::RBrace),
                '['  => self.ok_token(TokenData::LBracket),
                ']'  => self.ok_token(TokenData::RBracket),
                ';'  => self.ok_token(TokenData::Semicolon),
                ':'  => self.try_next(':', || TokenData::ColonColon, || TokenData::Colon),
                '.'  => self.ok_token(TokenData::Dot),
                ','  => self.ok_token(TokenData::Comma),
                '+'  => self.try_next('+', || TokenData::Increment,  || TokenData::Plus),
                '-'  => self.try_next('-', || TokenData::Decrement,  || TokenData::Minus),
                '*'  => self.try_next('*', || TokenData::StarStar,   || TokenData::Star),
                '/'  => self.try_next('/', || TokenData::SlashSlash, || TokenData::Slash),
                '%'  => self.ok_token(TokenData::Percent),
                '&'  => self.try_next('&', || TokenData::AndAnd,     || TokenData::And),
                '|'  => self.try_next('|', || TokenData::OrOr,       || TokenData::Or),
                '^'  => self.try_next('^', || TokenData::XorXor,     || TokenData::Xor),
                '!'  => self.try_next('=', || TokenData::Neq,        || TokenData::Not),
                '<'  => self.try_next('=', || TokenData::Le,         || TokenData::Lt),
                '>'  => self.try_next('=', || TokenData::Ge,         || TokenData::Gt),
                '\n' => self.ok_token(TokenData::NewLine),
                '='  => {
                    let token =
                        if let Some(char) = self.input.chars().nth(self.pos) {
                            match char {
                                '=' => TokenData::Eq,
                                '+' => TokenData::AssignPlus,
                                '-' => TokenData::AssignMinus,
                                '*' => TokenData::AssignStar,
                                '/' => TokenData::AssignSlash,
                                '%' => TokenData::AssignPercent,
                                '&' => TokenData::AssignAnd,
                                '|' => TokenData::AssignOr,
                                '^' => TokenData::AssignXor,
                                '!' => TokenData::AssignNot,
                                '<' => if self.input.chars().nth(self.pos + 1) == Some('=') { self.inc_pos(); TokenData::AssignLe } else { TokenData::AssignLt },
                                '>' => if self.input.chars().nth(self.pos + 1) == Some('=') { self.inc_pos(); TokenData::AssignGe } else { TokenData::AssignGt },
                                _   => TokenData::Assign
                            }
                        } else {
                            TokenData::Assign
                        };
                    if token != TokenData::Assign { self.inc_pos() }
                    self.ok_token(token)
                },
                '\"' => {
                    let mut text = vec![];
                    loop {
                        if let Some(char) = self.input.chars().nth(self.pos) {
                            self.inc_pos();
                            match char {
                                '\\' => {
                                    if let Some(char) = self.input.chars().nth(self.pos) {
                                        self.inc_pos();
                                        match char {
                                            '\\' => text.push('\\'),
                                            '\'' => text.push('\''),
                                            '\"' => text.push('\"'),
                                            'a' => text.push('\x07'),
                                            'b' => text.push('\x08'),
                                            't' => text.push('\t'),
                                            'n' => text.push('\n'),
                                            'f' => text.push('\x0C'),
                                            'r' => text.push('\r'),
                                            _ => return self.err_string_format()
                                        }
                                    } else { return self.err_string_format() }
                                }
                                '"' => break,
                                _ => text.push(char)
                            }
                        } else { return self.err_string_format() }
                    }
                    let text = text.iter().collect::<String>();
                    self.ok_token(TokenData::String(text))
                },
                _    => {
                    if char.is_numeric() {
                        self.next_number(char)
                    } else if char.is_alphabetic() || char == '_' {
                        let mut identifier = vec![char];
                        loop {
                            if let Some(char) = self.input.chars().nth(self.pos) && (char.is_alphanumeric() || char == '_' || char == '/') {
                                identifier.push(char);
                                self.inc_pos();
                            } else {
                                break
                            }
                        }
                        let identifier = identifier.iter().collect::<String>();
                        match identifier.as_str() {
                            "module"    => self.ok_token(TokenData::IdModule),
                            "class"     => self.ok_token(TokenData::IdClass),
                            "object"    => self.ok_token(TokenData::IdObject),
                            "extends"   => self.ok_token(TokenData::IdExtends),
                            "and"       => self.ok_token(TokenData::IdAnd),
                            "fun"       => self.ok_token(TokenData::IdFun),
                            "def"       => self.ok_token(TokenData::IdDef),
                            "use"       => self.ok_token(TokenData::IdUse),
                            "as"        => self.ok_token(TokenData::IdAs),
                            "block"     => self.ok_token(TokenData::IdBlock),
                            "if"        => self.ok_token(TokenData::IdIf),
                            "else"      => self.ok_token(TokenData::IdElse),
                            "while"     => self.ok_token(TokenData::IdWhile),
                            "do"        => self.ok_token(TokenData::IdDo),
                            "for"       => self.ok_token(TokenData::IdFor),
                            "in"        => self.ok_token(TokenData::IdIn),
                            "try"       => self.ok_token(TokenData::IdTry),
                            "catch"     => self.ok_token(TokenData::IdCatch),
                            "new"       => self.ok_token(TokenData::IdNew),
                            "let"       => self.ok_token(TokenData::IdLet),
                            "return"    => self.ok_token(TokenData::IdReturn),
                            "throw"     => self.ok_token(TokenData::IdThrow),
                            "end"       => self.ok_token(TokenData::IdEnd),
                            "true"      => self.ok_token(TokenData::Boolean(true)),
                            "false"     => self.ok_token(TokenData::Boolean(false)),
                            _           => self.ok_token(TokenData::Ident(identifier))
                        }
                    } else {
                        self.err_unexpected(char)
                    }
                }
            }
        } else {
            Ok(None)
        }
    }
    
    fn next_number(&mut self, char: char) -> Result<Option<Token>, LexerError> {
        let mut number = vec![char];
        let mut is_double = false;
        loop {
            if let Some(char) = self.input.chars().nth(self.pos) {
                if !char.is_numeric() {
                    if char != '.' { break }
                    if is_double { return self.err_number_format() }
                    is_double = true;
                }
                number.push(char);
                self.inc_pos();
            } else {
                break
            }
        }
        let number = number.iter().collect::<String>();
        if is_double {
            if let Ok(number) = f64::from_str(&number) {
                return self.ok_token(TokenData::Double(number))
            }
        } else {
            if let Ok(number) = i64::from_str(&number) {
                return self.ok_token(TokenData::Integer(number))
            }
        }
        self.err_number_format()
    }

    fn try_next(&mut self, cond: char, then: fn () -> TokenData, r#else: fn () -> TokenData) -> Result<Option<Token>, LexerError> {
        let token =
            if self.input.chars().nth(self.pos) == Some(cond) {
                self.inc_pos();
                then()
            } else {
                r#else()
            };
        self.ok_token(token)
    }

    #[inline(always)]
    fn err_string_format(&mut self) -> Result<Option<Token>, LexerError> {
        Err(LexerError::StringFormatError(self.file.to_owned(), self.line, self.column))
    }

    #[inline(always)]
    fn err_number_format(&mut self) -> Result<Option<Token>, LexerError> {
        Err(LexerError::NumberFormatError(self.file.to_owned(), self.line, self.column))
    }

    #[inline(always)]
    fn err_unexpected(&mut self, char: char) -> Result<Option<Token>, LexerError> {
        Err(LexerError::UnexpectedSymbol(self.file.to_owned(), self.line, self.column, char))
    }

    #[inline(always)]
    fn ok_token(&mut self, data: TokenData) -> Result<Option<Token>, LexerError> {
        Ok(Some(Token { data, debug: TokenDebug { line: self.line, column: self.column } }))
    }

    #[inline(always)]
    fn skip_ws(&mut self) {
        while let Some(c) = self.input.chars().nth(self.pos) && c.is_whitespace() && c != '\n' {
            self.inc_pos();
        }
    }

    fn inc_pos(&mut self) {
        if self.input.chars().nth(self.pos) == Some('\n') {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
        self.pos += 1;
    }
}