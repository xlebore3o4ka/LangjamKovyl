//! Синтаксический анализатор языка YG.
//!
//! Этот модуль содержит рекурсивный парсер, построенный по методу рекурсивного спуска
//! с поддержкой приоритетов операторов (Pratt parsing). Он преобразует поток токенов
//! от лексера в абстрактное синтаксическое дерево (AST) с сохранением информации
//! о позициях для диагностики ошибок.
//!
//! # Основные возможности
//!
//! - Модули, функции, классы, объекты.
//! - Наследование с множественным расширением (`extends ... and ...`).
//! - Разрешение имён через `use` и алиасы.
//! - Автоматическая квалификация имён текущим модулем для классов/объектов.
//! - Статические вызовы (`Class:method(...)`).
//! - Вызовы функций из модулей (`Module::function(...)`) – требуют явного импорта алиаса.
//! - Ссылки на объекты (`&X`).
//! - Поддержка `this::function` для вызова функций текущего модуля.
//! - Сохранение списка импортированных модулей в AST.

use crate::syntax::lexer::{Lexer, LexerError, Token, TokenData, TokenDebug};
use std::collections::HashMap;
use thiserror::Error;

// -----------------------------------------------------------------------------
// Ошибки парсинга
// -----------------------------------------------------------------------------

/// Ошибки, возникающие при синтаксическом анализе.
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("[{0} / {1}, {2}] Unexpected token: expected {3}, found {4:?}")]
    UnexpectedToken(String, u32, u32, String, TokenData),
    #[error("[{0} / {1}, {2}] Expected identifier")]
    ExpectedIdent(String, u32, u32),
    #[error("[{0} / {1}, {2}] Expected expression")]
    ExpectedExpr(String, u32, u32),
    #[error("[{0} / {1}, {2}] Invalid assignment target")]
    InvalidAssignTarget(String, u32, u32),
    #[error("[{0} / {1}, {2}] Unexpected end of file")]
    UnexpectedEOF(String, u32, u32),
    #[error("[{0} / {1}, {2}] Unexpected token: {3:?}")]
    UnexpectedTokenRaw(String, u32, u32, TokenData),
    #[error("[{0} / {1}, {2}] Unknown alias in path: {3}")]
    UnknownAlias(String, u32, u32, String),
    #[error("[{0} / {1}, {2}] 'this' used outside of module context")]
    ThisOutsideModule(String, u32, u32),
    #[error("{0}")]
    LexerError(#[from] LexerError),
}

// -----------------------------------------------------------------------------
// Абстрактное синтаксическое дерево (AST)
// -----------------------------------------------------------------------------

/// Корневой узел программы – модуль.
#[derive(Debug, Clone)]
pub struct Module {
    /// Полный путь к модулю (например, `std/io`).
    pub path: String,
    /// Список импортированных путей (из `use`).
    pub uses: Vec<String>,
    /// Элементы верхнего уровня (функции, классы, объекты).
    pub items: Vec<Item>,
}

/// Элемент верхнего уровня модуля.
#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Class(Class),
    Object(Class),
}

/// Определение функции.
#[derive(Debug, Clone)]
pub struct Function {
    /// Имя (пустая строка для анонимных функций).
    pub name: String,
    /// Список захваченных переменных (Some(vec![]) – пустой список,
    /// None – захват всех переменных (`[*]`)).
    pub captures: Option<Vec<String>>,
    /// Параметры функции.
    pub params: Vec<String>,
    /// Тело функции (блок операторов).
    pub body: Block,
}

/// Блок операторов (последовательность выражений/инструкций).
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// Оператор (инструкция).
#[derive(Debug, Clone)]
pub enum Statement {
    Let(Let),
    Assign(Assign),
    If(If),
    While(While),
    For(For),
    Expr(Expr),
    Return(Option<Expr>),
    Block(Block),
    TryCatch(TryCatch),
    Throw(Expr),
}

/// Объявление локальной переменной: `let name = value`.
#[derive(Debug, Clone)]
pub struct Let {
    pub name: String,
    pub value: Expr,
}

/// Присваивание.
#[derive(Debug, Clone)]
pub struct Assign {
    pub target: AssignTarget,
    pub op: AssignOp,
    pub value: Expr,
}

/// Цель присваивания (переменная, индекс, поле).
#[derive(Debug, Clone)]
pub enum AssignTarget {
    Var(String),
    Index { target: Box<Expr>, index: Box<Expr> },
    Member { target: Box<Expr>, name: String },
}

/// Операторы присваивания с комбинированными действиями.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,     // =
    PlusEq,     // =+
    MinusEq,    // =-
    StarEq,     // =*
    SlashEq,    // =/
    PercentEq,  // =%
    AndEq,      // =&
    OrEq,       // =|
    XorEq,      // =^
    NotEq,      // =!
    LtEq,       // =<
    GtEq,       // =>
    LeEq,       // =<=
    GeEq,       // =>=
}

/// Условный оператор `if`.
#[derive(Debug, Clone)]
pub struct If {
    pub cond: Expr,
    pub then_block: Block,
    pub else_block: Option<Block>,
}

/// Цикл `while`.
#[derive(Debug, Clone)]
pub struct While {
    pub cond: Expr,
    pub body: Block,
}

/// Цикл `for`.
#[derive(Debug, Clone)]
pub struct For {
    pub var: String,
    pub iter: Expr,
    pub body: Block,
}

/// Конструкция `try ... catch`.
#[derive(Debug, Clone)]
pub struct TryCatch {
    pub try_block: Block,
    pub catch_param: String,
    pub catch_block: Block,
}

/// Выражения.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Var(String),
    Binary(Binary),
    Unary(Unary),
    Call(Call),
    Index(Index),
    Member(Member),
    MethodCallColon(MethodCallColon),
    MethodCallDot(MethodCallDot),
    ModuleCall(ModuleCall),
    New(New),
    Function(Function),
    /// Ссылка на объект: `&X`.
    ObjectRef(String),
}

/// Литералы.
#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Double(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
}

/// Бинарная операция.
#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>,
}

/// Бинарные операторы.
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    And, Or, Xor,
    Eq, Neq, Lt, Gt, Le, Ge,
}

/// Унарная операция.
#[derive(Debug, Clone)]
pub struct Unary {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
}

/// Унарные операторы.
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Вызов функции.
#[derive(Debug, Clone)]
pub struct Call {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

/// Индексация (доступ по индексу).
#[derive(Debug, Clone)]
pub struct Index {
    pub target: Box<Expr>,
    pub index: Box<Expr>,
}

/// Доступ к полю (через `.`).
#[derive(Debug, Clone)]
pub struct Member {
    pub target: Box<Expr>,
    pub name: String,
}

/// Статический вызов метода: `Class:method(...)`.
#[derive(Debug, Clone)]
pub struct MethodCallColon {
    pub class: String,
    pub method: String,
    pub args: Vec<Expr>,
}

/// Вызов метода экземпляра: `obj.method(...)`.
#[derive(Debug, Clone)]
pub struct MethodCallDot {
    pub target: Box<Expr>,
    pub method: String,
    pub args: Vec<Expr>,
}

/// Вызов функции из модуля: `Module::function(...)`.
#[derive(Debug, Clone)]
pub struct ModuleCall {
    pub module: String,
    pub function: String,
    pub args: Vec<Expr>,
}

/// Создание экземпляра класса: `new Class(...)`.
#[derive(Debug, Clone)]
pub struct New {
    pub class: String,
    pub args: Vec<Expr>,
}

/// Определение класса.
#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub extends: Vec<String>,
    pub methods: Vec<Method>,
}

/// Метод класса/объекта.
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub params: Vec<String>,
    pub body: Block,
}

// -----------------------------------------------------------------------------
// Парсер
// -----------------------------------------------------------------------------

/// Рекурсивный парсер с поддержкой Pratt.
pub struct Parser {
    lexer: Lexer,
    file: String,
    current: Option<Token>,
    last_debug: Option<TokenDebug>,
    module_path: Option<String>,
    use_map: HashMap<String, String>,
    uses: Vec<String>,
}

impl Parser {
    /// Создаёт новый парсер из лексера.
    pub fn new(mut lexer: Lexer) -> Self {
        let file = lexer.file().clone();
        let current = lexer.next().unwrap_or(None);
        let last_debug = current.as_ref().map(|t| t.debug);
        Parser {
            lexer,
            file,
            current,
            last_debug,
            module_path: None,
            use_map: HashMap::new(),
            uses: Vec::new(),
        }
    }

    // ---- Вспомогательные методы ----

    fn current_debug(&self) -> TokenDebug {
        self.current
            .as_ref()
            .map(|t| t.debug)
            .unwrap_or_else(|| self.last_debug.unwrap_or(TokenDebug { line: 0, column: 0 }))
    }

    fn next_token(&mut self) -> Result<(), ParserError> {
        self.current = self.lexer.next()?;
        if let Some(ref tok) = self.current {
            self.last_debug = Some(tok.debug);
        }
        Ok(())
    }

    fn peek_data(&self) -> Option<&TokenData> {
        self.current.as_ref().map(|t| &t.data)
    }

    fn match_token(&mut self, expected: &TokenData) -> bool {
        if self.peek_data() == Some(expected) {
            self.next_token().unwrap_or(());
            true
        } else {
            false
        }
    }

    fn expect_token(&mut self, expected: TokenData) -> Result<(), ParserError> {
        if self.match_token(&expected) {
            Ok(())
        } else {
            let debug = self.current_debug();
            let found = self.peek_data().cloned().unwrap_or(TokenData::IdEnd);
            Err(ParserError::UnexpectedToken(
                self.file.clone(),
                debug.line,
                debug.column,
                format!("{:?}", expected),
                found,
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParserError> {
        if let Some(TokenData::Ident(s)) = self.peek_data() {
            let s = s.clone();
            self.next_token()?;
            Ok(s)
        } else {
            let debug = self.current_debug();
            Err(ParserError::ExpectedIdent(self.file.clone(), debug.line, debug.column))
        }
    }

    /// Парсит путь, состоящий из идентификаторов, разделённых `::`.
    fn parse_path(&mut self) -> Result<String, ParserError> {
        let mut parts = Vec::new();
        let first = self.expect_ident()?;
        parts.push(first);
        while self.match_token(&TokenData::ColonColon) {
            let next = self.expect_ident()?;
            parts.push(next);
        }
        Ok(parts.join("::"))
    }

    /// Разрешает имя класса/объекта с учётом `use` и префикса модуля.
    ///
    /// # Правила разрешения
    /// 1. Если имя содержит `/`, оно уже квалифицировано и возвращается без изменений.
    /// 2. Если имя содержит `::`, префикс ищется в `use_map`:
    ///    - если найден – заменяется на путь (с `/`), суффикс добавляется через `/`.
    ///    - иначе – ошибка `UnknownAlias`.
    /// 3. Если имя не содержит ни `/`, ни `::`:
    ///    - если есть точное совпадение в `use_map` – возвращается путь из маппинга.
    ///    - иначе к имени добавляется префикс текущего модуля (через `/`).
    fn resolve_class_name(&self, name: &str) -> Result<String, ParserError> {
        if name.contains('/') {
            return Ok(name.to_string());
        }

        if let Some(colon_pos) = name.find("::") {
            let prefix = &name[..colon_pos];
            let suffix = &name[colon_pos + 2..];
            if let Some(path) = self.use_map.get(prefix) {
                return Ok(format!("{}/{}", path, suffix));
            }
            let debug = self.current_debug();
            return Err(ParserError::UnknownAlias(
                self.file.clone(),
                debug.line,
                debug.column,
                prefix.to_string(),
            ));
        }

        if let Some(path) = self.use_map.get(name) {
            Ok(path.clone())
        } else if let Some(module) = &self.module_path {
            Ok(format!("{}/{}", module, name))
        } else {
            Ok(name.to_string())
        }
    }

    /// Разрешает путь для вызова функции из модуля.
    /// Поддерживает `this::function` – заменяет `this` на путь текущего модуля.
    fn resolve_module_path(&self, path: &str) -> Result<String, ParserError> {
        let mut full_path = path.to_string();
        // Если начинается с "this::", заменяем "this" на путь модуля
        if full_path.starts_with("this::") {
            if let Some(module_path) = &self.module_path {
                full_path = format!("{}::{}", module_path, &full_path[6..]); // 6 = len("this::")
            } else {
                let debug = self.current_debug();
                return Err(ParserError::ThisOutsideModule(
                    self.file.clone(),
                    debug.line,
                    debug.column,
                ));
            }
        }
        Ok(full_path)
    }

    fn skip_separators(&mut self) {
        while let Some(TokenData::NewLine) | Some(TokenData::Semicolon) = self.peek_data() {
            self.next_token().unwrap_or(());
        }
    }

    // ---- Парсинг модуля ----

    /// Разбирает весь модуль.
    pub fn parse_module(&mut self) -> Result<Module, ParserError> {
        self.expect_token(TokenData::IdModule)?;
        let path = self.expect_ident()?;
        self.module_path = Some(path.clone());
        let mut items = Vec::new();
        self.skip_separators();
        loop {
            match self.peek_data() {
                Some(TokenData::IdUse) => {
                    self.parse_use()?;
                }
                Some(TokenData::IdFun) => items.push(Item::Function(self.parse_function(true)?)),
                Some(TokenData::IdClass) => items.push(Item::Class(self.parse_class()?)),
                Some(TokenData::IdObject) => items.push(Item::Object(self.parse_object()?)),
                Some(TokenData::IdEnd) => {
                    self.next_token()?;
                    break;
                }
                Some(other) => {
                    let debug = self.current_debug();
                    return Err(ParserError::UnexpectedTokenRaw(
                        self.file.clone(),
                        debug.line,
                        debug.column,
                        other.clone(),
                    ));
                }
                None => {
                    let debug = self.current_debug();
                    return Err(ParserError::UnexpectedEOF(self.file.clone(), debug.line, debug.column));
                }
            }
            self.skip_separators();
        }
        Ok(Module {
            path,
            uses: std::mem::take(&mut self.uses),
            items,
        })
    }

    fn parse_use(&mut self) -> Result<(), ParserError> {
        self.expect_token(TokenData::IdUse)?;
        let path = self.expect_ident()?;
        self.uses.push(path.clone());
        let alias = if self.match_token(&TokenData::IdAs) {
            self.expect_ident()?
        } else {
            path.split('/').last().unwrap_or(&path).to_string()
        };
        self.use_map.insert(alias, path);
        Ok(())
    }

    // ---- Парсинг функций ----

    fn parse_function(&mut self, named: bool) -> Result<Function, ParserError> {
        self.expect_token(TokenData::IdFun)?;
        let name = if named { self.expect_ident()? } else { String::new() };

        let captures = if self.match_token(&TokenData::LBracket) {
            if self.match_token(&TokenData::Star) {
                self.expect_token(TokenData::RBracket)?;
                None
            } else {
                let mut caps = Vec::new();
                while let Some(TokenData::Ident(s)) = self.peek_data() {
                    caps.push(s.clone());
                    self.next_token()?;
                    if !self.match_token(&TokenData::Comma) {
                        break;
                    }
                }
                self.expect_token(TokenData::RBracket)?;
                Some(caps)
            }
        } else {
            Some(Vec::new())
        };

        self.expect_token(TokenData::LParen)?;
        let mut params = Vec::new();
        self.skip_separators();
        while let Some(TokenData::Ident(s)) = self.peek_data() {
            params.push(s.clone());
            self.next_token()?;
            self.skip_separators();
            if !self.match_token(&TokenData::Comma) {
                break;
            }
            self.skip_separators();
        }
        self.expect_token(TokenData::RParen)?;
        let body = self.parse_body()?;
        Ok(Function {
            name,
            captures,
            params,
            body,
        })
    }

    // ---- Парсинг тела ----

    fn parse_body(&mut self) -> Result<Block, ParserError> {
        let mut statements = Vec::new();
        self.skip_separators();
        while let Some(tok) = self.peek_data() {
            if *tok == TokenData::IdEnd {
                self.next_token()?;
                break;
            }
            statements.push(self.parse_statement()?);
            self.skip_separators();
        }
        Ok(Block { statements })
    }

    // ---- Парсинг операторов ----

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.peek_data() {
            Some(TokenData::IdLet) => {
                self.next_token()?;
                self.parse_let()
            }
            Some(TokenData::IdIf) => self.parse_if(),
            Some(TokenData::IdWhile) => self.parse_while(),
            Some(TokenData::IdFor) => self.parse_for(),
            Some(TokenData::IdReturn) => self.parse_return(),
            Some(TokenData::IdTry) => self.parse_try_catch(),
            Some(TokenData::IdThrow) => self.parse_throw(),
            Some(TokenData::IdBlock) => {
                self.next_token()?;
                let block = self.parse_body()?;
                self.skip_separators();
                Ok(Statement::Block(block))
            }
            _ => {
                let expr = self.parse_expr(0)?;
                if let Some(op) = self.peek_assign_op() {
                    let op = op.clone();
                    self.next_token()?;
                    let value = self.parse_expr(0)?;
                    let target = match expr {
                        Expr::Var(name) => AssignTarget::Var(name),
                        Expr::Index(idx) => AssignTarget::Index {
                            target: idx.target,
                            index: idx.index,
                        },
                        Expr::Member(mem) => AssignTarget::Member {
                            target: mem.target,
                            name: mem.name,
                        },
                        _ => {
                            let debug = self.current_debug();
                            return Err(ParserError::InvalidAssignTarget(
                                self.file.clone(),
                                debug.line,
                                debug.column,
                            ));
                        }
                    };
                    Ok(Statement::Assign(Assign { target, op, value }))
                } else {
                    Ok(Statement::Expr(expr))
                }
            }
        }
    }

    fn peek_assign_op(&self) -> Option<AssignOp> {
        match self.peek_data() {
            Some(TokenData::Assign) => Some(AssignOp::Assign),
            Some(TokenData::AssignPlus) => Some(AssignOp::PlusEq),
            Some(TokenData::AssignMinus) => Some(AssignOp::MinusEq),
            Some(TokenData::AssignStar) => Some(AssignOp::StarEq),
            Some(TokenData::AssignSlash) => Some(AssignOp::SlashEq),
            Some(TokenData::AssignPercent) => Some(AssignOp::PercentEq),
            Some(TokenData::AssignAnd) => Some(AssignOp::AndEq),
            Some(TokenData::AssignOr) => Some(AssignOp::OrEq),
            Some(TokenData::AssignXor) => Some(AssignOp::XorEq),
            Some(TokenData::AssignNot) => Some(AssignOp::NotEq),
            Some(TokenData::AssignLt) => Some(AssignOp::LtEq),
            Some(TokenData::AssignGt) => Some(AssignOp::GtEq),
            Some(TokenData::AssignLe) => Some(AssignOp::LeEq),
            Some(TokenData::AssignGe) => Some(AssignOp::GeEq),
            _ => None,
        }
    }

    fn parse_let(&mut self) -> Result<Statement, ParserError> {
        let name = self.expect_ident()?;
        self.expect_token(TokenData::Assign)?;
        let value = self.parse_expr(0)?;
        Ok(Statement::Let(Let { name, value }))
    }

    fn parse_if(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(TokenData::IdIf)?;
        self.expect_token(TokenData::LParen)?;
        let cond = self.parse_expr(0)?;
        self.expect_token(TokenData::RParen)?;

        let mut then_stmts = Vec::new();
        self.skip_separators();
        while let Some(tok) = self.peek_data() {
            if *tok == TokenData::IdElse || *tok == TokenData::IdEnd {
                break;
            }
            then_stmts.push(self.parse_statement()?);
            self.skip_separators();
        }
        let then_block = Block {
            statements: then_stmts,
        };

        let else_block = if self.match_token(&TokenData::IdElse) {
            let mut else_stmts = Vec::new();
            self.skip_separators();
            while let Some(tok) = self.peek_data() {
                if *tok == TokenData::IdEnd {
                    break;
                }
                else_stmts.push(self.parse_statement()?);
                self.skip_separators();
            }
            Some(Block {
                statements: else_stmts,
            })
        } else {
            None
        };

        self.expect_token(TokenData::IdEnd)?;
        Ok(Statement::If(If {
            cond,
            then_block,
            else_block,
        }))
    }

    fn parse_while(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(TokenData::IdWhile)?;
        self.expect_token(TokenData::LParen)?;
        let cond = self.parse_expr(0)?;
        self.expect_token(TokenData::RParen)?;
        let body = self.parse_body()?;
        Ok(Statement::While(While { cond, body }))
    }

    fn parse_for(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(TokenData::IdFor)?;
        self.expect_token(TokenData::LParen)?;
        let var = self.expect_ident()?;
        self.expect_token(TokenData::IdIn)?;
        let iter = self.parse_expr(0)?;
        self.expect_token(TokenData::RParen)?;
        let body = self.parse_body()?;
        Ok(Statement::For(For { var, iter, body }))
    }

    fn parse_return(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(TokenData::IdReturn)?;
        let value = if let Some(tok) = self.peek_data() {
            if *tok != TokenData::IdEnd && *tok != TokenData::NewLine && *tok != TokenData::Semicolon {
                Some(self.parse_expr(0)?)
            } else {
                None
            }
        } else {
            None
        };
        Ok(Statement::Return(value))
    }

    // ---- try / catch / throw ----

    fn parse_try_catch(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(TokenData::IdTry)?;

        let mut try_stmts = Vec::new();
        self.skip_separators();
        while let Some(tok) = self.peek_data() {
            if *tok == TokenData::IdCatch || *tok == TokenData::IdEnd {
                break;
            }
            try_stmts.push(self.parse_statement()?);
            self.skip_separators();
        }
        let try_block = Block {
            statements: try_stmts,
        };

        self.expect_token(TokenData::IdCatch)?;
        self.expect_token(TokenData::LParen)?;
        let catch_param = self.expect_ident()?;
        self.expect_token(TokenData::RParen)?;

        let mut catch_stmts = Vec::new();
        self.skip_separators();
        while let Some(tok) = self.peek_data() {
            if *tok == TokenData::IdEnd {
                break;
            }
            catch_stmts.push(self.parse_statement()?);
            self.skip_separators();
        }
        let catch_block = Block {
            statements: catch_stmts,
        };

        self.expect_token(TokenData::IdEnd)?;
        Ok(Statement::TryCatch(TryCatch {
            try_block,
            catch_param,
            catch_block,
        }))
    }

    fn parse_throw(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(TokenData::IdThrow)?;
        let expr = self.parse_expr(0)?;
        Ok(Statement::Throw(expr))
    }

    // ---- Выражения (Pratt) ----

    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, ParserError> {
        let mut left = self.parse_primary()?;
        while let Some((op, prec)) = self.peek_binary_op() {
            if prec < min_prec {
                break;
            }
            self.next_token()?;
            let right = self.parse_expr(prec + 1)?;
            left = Expr::Binary(Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    /// Возвращает бинарный оператор и его приоритет.
    /// Теперь бинарные операторы используют двойные символы: &&, ||, ^^.
    /// Одиночный & используется только как унарная объектная ссылка.
    fn peek_binary_op(&self) -> Option<(BinaryOp, u8)> {
        match self.peek_data() {
            Some(TokenData::Plus) => Some((BinaryOp::Add, 4)),
            Some(TokenData::Minus) => Some((BinaryOp::Sub, 4)),
            Some(TokenData::Star) => Some((BinaryOp::Mul, 5)),
            Some(TokenData::Slash) => Some((BinaryOp::Div, 5)),
            Some(TokenData::Percent) => Some((BinaryOp::Mod, 5)),
            Some(TokenData::Eq) => Some((BinaryOp::Eq, 3)),
            Some(TokenData::Neq) => Some((BinaryOp::Neq, 3)),
            Some(TokenData::Lt) => Some((BinaryOp::Lt, 3)),
            Some(TokenData::Gt) => Some((BinaryOp::Gt, 3)),
            Some(TokenData::Le) => Some((BinaryOp::Le, 3)),
            Some(TokenData::Ge) => Some((BinaryOp::Ge, 3)),
            // ВНИМАНИЕ: вместо одиночных &, |, ^ теперь используются двойные:
            Some(TokenData::AndAnd) => Some((BinaryOp::And, 2)),   // &&
            Some(TokenData::OrOr)   => Some((BinaryOp::Or, 1)),    // ||
            Some(TokenData::XorXor) => Some((BinaryOp::Xor, 2)),   // ^^
            _ => None,
        }
    }

    // ---- Первичные выражения ----

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        let mut expr = match self.peek_data() {
            Some(TokenData::Integer(n)) => {
                let val = *n;
                self.next_token()?;
                Expr::Literal(Literal::Integer(val))
            }
            Some(TokenData::Double(n)) => {
                let val = *n;
                self.next_token()?;
                Expr::Literal(Literal::Double(val))
            }
            Some(TokenData::String(s)) => {
                let val = s.clone();
                self.next_token()?;
                Expr::Literal(Literal::String(val))
            }
            Some(TokenData::Boolean(b)) => {
                let val = *b;
                self.next_token()?;
                Expr::Literal(Literal::Boolean(val))
            }
            Some(TokenData::LBracket) => self.parse_array()?,
            Some(TokenData::LBrace) => self.parse_map()?,
            Some(TokenData::LParen) => {
                self.next_token()?;
                self.skip_separators();
                let expr = self.parse_expr(0)?;
                self.expect_token(TokenData::RParen)?;
                expr
            }
            Some(TokenData::IdNew) => self.parse_new()?,
            Some(TokenData::Ident(s)) => {
                let ident = s.clone();
                self.next_token()?;
                let mut path_parts = vec![ident];
                while self.match_token(&TokenData::ColonColon) {
                    let next = self.expect_ident()?;
                    path_parts.push(next);
                }
                let full_name = path_parts.join("::");

                if self.match_token(&TokenData::Colon) {
                    let method = self.expect_ident()?;
                    let args = self.parse_args()?;
                    let class = self.resolve_class_name(&full_name)?;
                    Expr::MethodCallColon(MethodCallColon {
                        class,
                        method,
                        args,
                    })
                } else {
                    Expr::Var(full_name)
                }
            }
            Some(TokenData::IdFun) => {
                let func = self.parse_function(false)?;
                Expr::Function(func)
            }
            Some(TokenData::Minus) => {
                self.next_token()?;
                let expr = self.parse_expr(6)?;
                Expr::Unary(Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            Some(TokenData::Not) => {
                self.next_token()?;
                let expr = self.parse_expr(6)?;
                Expr::Unary(Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            // Ссылка на объект: &X
            // Здесь используется одиночный &, который теперь НЕ является бинарным оператором.
            Some(TokenData::And) => {
                self.next_token()?;
                let path_str = self.parse_path()?;
                let resolved_object = self.resolve_class_name(&path_str)?;
                // Проверяем, не идёт ли за ним ':' – это была бы попытка использовать &X:F, что больше не поддерживается
                if self.match_token(&TokenData::Colon) {
                    let debug = self.current_debug();
                    return Err(ParserError::UnexpectedTokenRaw(
                        self.file.clone(),
                        debug.line,
                        debug.column,
                        TokenData::Colon,
                    ));
                }
                Expr::ObjectRef(resolved_object)
            }
            Some(other) => {
                let debug = self.current_debug();
                return Err(ParserError::UnexpectedTokenRaw(
                    self.file.clone(),
                    debug.line,
                    debug.column,
                    other.clone(),
                ));
            }
            None => {
                let debug = self.current_debug();
                return Err(ParserError::UnexpectedEOF(self.file.clone(), debug.line, debug.column));
            }
        };

        loop {
            self.skip_separators();
            match self.peek_data() {
                Some(TokenData::Dot) => {
                    self.next_token()?;
                    let member = self.expect_ident()?;
                    expr = Expr::Member(Member {
                        target: Box::new(expr),
                        name: member,
                    });
                }
                Some(TokenData::LBracket) => {
                    self.next_token()?;
                    let index = self.parse_expr(0)?;
                    self.expect_token(TokenData::RBracket)?;
                    expr = Expr::Index(Index {
                        target: Box::new(expr),
                        index: Box::new(index),
                    });
                }
                Some(TokenData::LParen) => {
                    self.next_token()?;
                    let args = self.parse_args_after_paren()?;
                    // Определяем вид вызова по текущему выражению.
                    if let Expr::Member(m) = expr {
                        expr = Expr::MethodCallDot(MethodCallDot {
                            target: m.target,
                            method: m.name,
                            args,
                        });
                    } else if let Expr::Var(name) = &expr {
                        // Если имя содержит `::`, это вызов функции из модуля.
                        if name.contains("::") {
                            // Разрешаем `this::` в путь модуля
                            let resolved_name = self.resolve_module_path(name)?;
                            let parts: Vec<&str> = resolved_name.split("::").collect();
                            if parts.len() < 2 {
                                // не должно случиться
                                let debug = self.current_debug();
                                return Err(ParserError::UnexpectedTokenRaw(
                                    self.file.clone(),
                                    debug.line,
                                    debug.column,
                                    TokenData::Ident(resolved_name.clone()),
                                ));
                            }
                            let function = parts.last().unwrap().to_string();
                            let prefix_parts = &parts[..parts.len()-1];
                            // Разрешаем первый компонент как алиас
                            let first = prefix_parts[0];
                            let base_path = if let Some(path) = self.use_map.get(first) {
                                path.clone()
                            } else if first.contains('/') {
                                first.to_string()
                            } else if self.module_path.as_deref() == Some(first) {
                                // если первый компонент — это имя текущего модуля
                                first.to_string()
                            } else {
                                let debug = self.current_debug();
                                return Err(ParserError::UnknownAlias(
                                    self.file.clone(),
                                    debug.line,
                                    debug.column,
                                    first.to_string(),
                                ));
                            };
                            // Добавляем остальные компоненты через `/`
                            let mut module_path = base_path;
                            for &part in &prefix_parts[1..] {
                                module_path = format!("{}/{}", module_path, part);
                            }
                            expr = Expr::ModuleCall(ModuleCall {
                                module: module_path,
                                function,
                                args,
                            });
                        } else {
                            expr = Expr::Call(Call {
                                callee: Box::new(expr),
                                args,
                            });
                        }
                    } else {
                        expr = Expr::Call(Call {
                            callee: Box::new(expr),
                            args,
                        });
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    // ---- Массивы, словари, вызовы ----

    fn parse_array(&mut self) -> Result<Expr, ParserError> {
        self.expect_token(TokenData::LBracket)?;
        let mut elements = Vec::new();
        self.skip_separators();
        while !self.match_token(&TokenData::RBracket) {
            elements.push(self.parse_expr(0)?);
            self.skip_separators();
            if !self.match_token(&TokenData::Comma) {
                self.expect_token(TokenData::RBracket)?;
                break;
            }
            self.skip_separators();
        }
        Ok(Expr::Literal(Literal::Array(elements)))
    }

    fn parse_map(&mut self) -> Result<Expr, ParserError> {
        self.expect_token(TokenData::LBrace)?;
        let mut pairs = Vec::new();
        self.skip_separators();
        while !self.match_token(&TokenData::RBrace) {
            let key = self.parse_expr(0)?;
            self.expect_token(TokenData::Colon)?;
            let value = self.parse_expr(0)?;
            pairs.push((key, value));
            self.skip_separators();
            if !self.match_token(&TokenData::Comma) {
                self.expect_token(TokenData::RBrace)?;
                break;
            }
            self.skip_separators();
        }
        Ok(Expr::Literal(Literal::Map(pairs)))
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParserError> {
        self.expect_token(TokenData::LParen)?;
        self.parse_args_after_paren()
    }

    fn parse_args_after_paren(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut args = Vec::new();
        self.skip_separators();
        loop {
            if self.match_token(&TokenData::RParen) {
                break;
            }
            args.push(self.parse_expr(0)?);
            self.skip_separators();
            if self.match_token(&TokenData::Comma) {
                self.skip_separators();
                continue;
            } else {
                self.expect_token(TokenData::RParen)?;
                break;
            }
        }
        Ok(args)
    }

    // ---- new, классы, объекты, методы ----

    fn parse_new(&mut self) -> Result<Expr, ParserError> {
        self.expect_token(TokenData::IdNew)?;
        let class_name = self.parse_path()?;
        let class = self.resolve_class_name(&class_name)?;
        let args = self.parse_args()?;
        Ok(Expr::New(New { class, args }))
    }

    fn parse_class(&mut self) -> Result<Class, ParserError> {
        self.expect_token(TokenData::IdClass)?;
        let name = self.expect_ident()?;
        let mut extends = Vec::new();
        if self.match_token(&TokenData::IdExtends) {
            let first = self.parse_path()?;
            extends.push(self.resolve_class_name(&first)?);
            while self.match_token(&TokenData::IdAnd) {
                let base = self.parse_path()?;
                extends.push(self.resolve_class_name(&base)?);
            }
        }
        let mut methods = Vec::new();
        self.skip_separators();
        while let Some(tok) = self.peek_data() {
            if *tok == TokenData::IdEnd {
                self.next_token()?;
                break;
            }
            if *tok == TokenData::IdDef {
                methods.push(self.parse_method()?);
            } else {
                let debug = self.current_debug();
                return Err(ParserError::UnexpectedTokenRaw(
                    self.file.clone(),
                    debug.line,
                    debug.column,
                    tok.clone(),
                ));
            }
            self.skip_separators();
        }
        Ok(Class {
            name,
            extends,
            methods,
        })
    }

    fn parse_object(&mut self) -> Result<Class, ParserError> {
        self.expect_token(TokenData::IdObject)?;
        let name = self.expect_ident()?;
        let mut extends = Vec::new();
        if self.match_token(&TokenData::IdExtends) {
            let first = self.parse_path()?;
            extends.push(self.resolve_class_name(&first)?);
            while self.match_token(&TokenData::IdAnd) {
                let base = self.parse_path()?;
                extends.push(self.resolve_class_name(&base)?);
            }
        }
        let mut methods = Vec::new();
        self.skip_separators();
        while let Some(tok) = self.peek_data() {
            if *tok == TokenData::IdEnd {
                self.next_token()?;
                break;
            }
            if *tok == TokenData::IdDef {
                methods.push(self.parse_method()?);
            } else {
                let debug = self.current_debug();
                return Err(ParserError::UnexpectedTokenRaw(
                    self.file.clone(),
                    debug.line,
                    debug.column,
                    tok.clone(),
                ));
            }
            self.skip_separators();
        }
        Ok(Class {
            name,
            extends,
            methods,
        })
    }

    fn parse_method(&mut self) -> Result<Method, ParserError> {
        self.expect_token(TokenData::IdDef)?;
        let name = self.expect_ident()?;
        self.expect_token(TokenData::LParen)?;
        let mut params = Vec::new();
        self.skip_separators();
        while let Some(TokenData::Ident(s)) = self.peek_data() {
            params.push(s.clone());
            self.next_token()?;
            self.skip_separators();
            if !self.match_token(&TokenData::Comma) {
                break;
            }
            self.skip_separators();
        }
        self.expect_token(TokenData::RParen)?;
        let body = self.parse_body()?;
        Ok(Method { name, params, body })
    }
}

// -----------------------------------------------------------------------------
// Тесты
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::lexer::Lexer;

    fn parse_test(src: &str) -> Module {
        let trimmed = src.trim();
        let full = if trimmed.starts_with("module") {
            trimmed.to_string()
        } else {
            format!("module test\n{}\nend", trimmed)
        };
        let lexer = Lexer::new("full.yg".to_string(), full);
        let mut parser = Parser::new(lexer);
        parser.parse_module().unwrap()
    }

    // ---- Базовые тесты (проверяем, что uses пуст) ----

    #[test]
    fn test_function_with_let() {
        let src = r#"
            fun foo()
                let x = 42
                let y = x + 1
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        assert_eq!(module.items.len(), 1);
        match &module.items[0] {
            Item::Function(func) => {
                assert_eq!(func.name, "foo");
                assert_eq!(func.params.len(), 0);
                assert_eq!(func.body.statements.len(), 2);
                match &func.body.statements[0] {
                    Statement::Let(let_stmt) => {
                        assert_eq!(let_stmt.name, "x");
                        match &let_stmt.value {
                            Expr::Literal(Literal::Integer(42)) => {}
                            _ => panic!("Expected integer literal 42"),
                        }
                    }
                    _ => panic!("Expected Let statement"),
                }
                match &func.body.statements[1] {
                    Statement::Let(let_stmt) => {
                        assert_eq!(let_stmt.name, "y");
                        match &let_stmt.value {
                            Expr::Binary(bin) => {
                                assert!(matches!(bin.op, BinaryOp::Add));
                                match bin.left.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "x"),
                                    _ => panic!("Expected variable x"),
                                }
                                match bin.right.as_ref() {
                                    Expr::Literal(Literal::Integer(1)) => {}
                                    _ => panic!("Expected integer 1"),
                                }
                            }
                            _ => panic!("Expected binary expression"),
                        }
                    }
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_if_statement() {
        let src = r#"
            fun test_if()
                let a = 5
                if (a < 10)
                    let b = 1
                else
                    let b = 2
                end
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 2);
                match &func.body.statements[1] {
                    Statement::If(if_stmt) => {
                        match &if_stmt.cond {
                            Expr::Binary(bin) => {
                                assert!(matches!(bin.op, BinaryOp::Lt));
                                match bin.left.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "a"),
                                    _ => panic!("Expected variable a"),
                                }
                                match bin.right.as_ref() {
                                    Expr::Literal(Literal::Integer(10)) => {}
                                    _ => panic!("Expected integer 10"),
                                }
                            }
                            _ => panic!("Expected binary expression"),
                        }
                        assert_eq!(if_stmt.then_block.statements.len(), 1);
                        assert!(matches!(if_stmt.then_block.statements[0], Statement::Let(_)));
                        assert!(if_stmt.else_block.is_some());
                        let else_block = if_stmt.else_block.as_ref().unwrap();
                        assert_eq!(else_block.statements.len(), 1);
                        assert!(matches!(else_block.statements[0], Statement::Let(_)));
                    }
                    _ => panic!("Expected If statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_while_loop() {
        let src = r#"
            fun test_while()
                let i = 0
                while (i < 10)
                    i =+ 1
                end
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 2);
                match &func.body.statements[1] {
                    Statement::While(while_stmt) => {
                        match &while_stmt.cond {
                            Expr::Binary(bin) => {
                                assert!(matches!(bin.op, BinaryOp::Lt));
                                match bin.left.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "i"),
                                    _ => panic!("Expected variable i"),
                                }
                                match bin.right.as_ref() {
                                    Expr::Literal(Literal::Integer(10)) => {}
                                    _ => panic!("Expected integer 10"),
                                }
                            }
                            _ => panic!("Expected binary expression"),
                        }
                        assert_eq!(while_stmt.body.statements.len(), 1);
                        match &while_stmt.body.statements[0] {
                            Statement::Assign(assign) => {
                                match &assign.target {
                                    AssignTarget::Var(name) => assert_eq!(name, "i"),
                                    _ => panic!("Expected variable target"),
                                }
                                assert!(matches!(assign.op, AssignOp::PlusEq));
                                match &assign.value {
                                    Expr::Literal(Literal::Integer(1)) => {}
                                    _ => panic!("Expected integer 1"),
                                }
                            }
                            _ => panic!("Expected Assign statement"),
                        }
                    }
                    _ => panic!("Expected While statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_for_loop() {
        let src = r#"
            fun test_for()
                for (i in range(0))
                    let x = i
                end
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::For(for_stmt) => {
                        assert_eq!(for_stmt.var, "i");
                        match &for_stmt.iter {
                            Expr::Call(call) => {
                                match call.callee.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "range"),
                                    _ => panic!("Expected callee range"),
                                }
                                assert_eq!(call.args.len(), 1);
                                match &call.args[0] {
                                    Expr::Literal(Literal::Integer(0)) => {}
                                    _ => panic!("Expected integer 0"),
                                }
                            }
                            _ => panic!("Expected call to range"),
                        }
                        assert_eq!(for_stmt.body.statements.len(), 1);
                        match &for_stmt.body.statements[0] {
                            Statement::Let(let_stmt) => {
                                assert_eq!(let_stmt.name, "x");
                                match &let_stmt.value {
                                    Expr::Var(name) => assert_eq!(name, "i"),
                                    _ => panic!("Expected variable i"),
                                }
                            }
                            _ => panic!("Expected Let statement"),
                        }
                    }
                    _ => panic!("Expected For statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_class_and_method() {
        let src = r#"
            use core
            class Foo extends core::Object
                def bar()
                    let x = 1
                end
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["core"]);
        match &module.items[0] {
            Item::Class(cls) => {
                assert_eq!(cls.name, "Foo");
                assert_eq!(cls.extends, vec!["core/Object"]);
                assert_eq!(cls.methods.len(), 1);
                let method = &cls.methods[0];
                assert_eq!(method.name, "bar");
                assert_eq!(method.params.len(), 0);
                assert_eq!(method.body.statements.len(), 1);
                match &method.body.statements[0] {
                    Statement::Let(let_stmt) => {
                        assert_eq!(let_stmt.name, "x");
                        match &let_stmt.value {
                            Expr::Literal(Literal::Integer(1)) => {}
                            _ => panic!("Expected integer 1"),
                        }
                    }
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected Class"),
        }
    }

    #[test]
    fn test_object() {
        let src = r#"
            use core
            object MyObject extends core::Object
                def init()
                    this.val = 42
                end
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["core"]);
        match &module.items[0] {
            Item::Object(obj) => {
                assert_eq!(obj.name, "MyObject");
                assert_eq!(obj.extends, vec!["core/Object"]);
                assert_eq!(obj.methods.len(), 1);
                let method = &obj.methods[0];
                assert_eq!(method.name, "init");
                assert_eq!(method.body.statements.len(), 1);
                match &method.body.statements[0] {
                    Statement::Assign(assign) => {
                        match &assign.target {
                            AssignTarget::Member { target, name } => {
                                match target.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "this"),
                                    _ => panic!("Expected this"),
                                }
                                assert_eq!(name, "val");
                            }
                            _ => panic!("Expected member assignment"),
                        }
                        assert!(matches!(assign.op, AssignOp::Assign));
                        match &assign.value {
                            Expr::Literal(Literal::Integer(42)) => {}
                            _ => panic!("Expected integer 42"),
                        }
                    }
                    _ => panic!("Expected Assign statement"),
                }
            }
            _ => panic!("Expected Object"),
        }
    }

    #[test]
    fn test_extends_with_use() {
        let src = r#"
            use core
            class Foo extends core::Object
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["core"]);
        match &module.items[0] {
            Item::Class(cls) => {
                assert_eq!(cls.extends, vec!["core/Object"]);
                assert_eq!(cls.name, "Foo");
            }
            _ => panic!("Expected Class"),
        }
    }

    #[test]
    fn test_binary_ops() {
        let src = r#"
            fun test_bin()
                let a = 1 + 2 * 3 - 4 / 2
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::Binary(bin) => {
                                assert!(matches!(bin.op, BinaryOp::Sub));
                                match bin.left.as_ref() {
                                    Expr::Binary(left_bin) => {
                                        assert!(matches!(left_bin.op, BinaryOp::Add));
                                        match left_bin.left.as_ref() {
                                            Expr::Literal(Literal::Integer(1)) => {}
                                            _ => panic!("Expected 1"),
                                        }
                                        match left_bin.right.as_ref() {
                                            Expr::Binary(mul_bin) => {
                                                assert!(matches!(mul_bin.op, BinaryOp::Mul));
                                                match mul_bin.left.as_ref() {
                                                    Expr::Literal(Literal::Integer(2)) => {}
                                                    _ => panic!("Expected 2"),
                                                }
                                                match mul_bin.right.as_ref() {
                                                    Expr::Literal(Literal::Integer(3)) => {}
                                                    _ => panic!("Expected 3"),
                                                }
                                            }
                                            _ => panic!("Expected multiplication"),
                                        }
                                    }
                                    _ => panic!("Expected addition"),
                                }
                                match bin.right.as_ref() {
                                    Expr::Binary(div_bin) => {
                                        assert!(matches!(div_bin.op, BinaryOp::Div));
                                        match div_bin.left.as_ref() {
                                            Expr::Literal(Literal::Integer(4)) => {}
                                            _ => panic!("Expected 4"),
                                        }
                                        match div_bin.right.as_ref() {
                                            Expr::Literal(Literal::Integer(2)) => {}
                                            _ => panic!("Expected 2"),
                                        }
                                    }
                                    _ => panic!("Expected division"),
                                }
                            }
                            _ => panic!("Expected binary expression"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_anon_function() {
        let src = r#"
            fun outer()
                let f = fun (x) x + 1 end
                f(5)
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmts = &func.body.statements;
                assert_eq!(stmts.len(), 2);
                match &stmts[0] {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::Function(anon) => {
                                assert_eq!(anon.name, "");
                                assert_eq!(anon.params, vec!["x"]);
                                assert_eq!(anon.body.statements.len(), 1);
                                match &anon.body.statements[0] {
                                    Statement::Expr(expr) => {
                                        match expr {
                                            Expr::Binary(bin) => {
                                                assert!(matches!(bin.op, BinaryOp::Add));
                                                match bin.left.as_ref() {
                                                    Expr::Var(name) => assert_eq!(name, "x"),
                                                    _ => panic!("Expected variable x"),
                                                }
                                                match bin.right.as_ref() {
                                                    Expr::Literal(Literal::Integer(1)) => {}
                                                    _ => panic!("Expected integer 1"),
                                                }
                                            }
                                            _ => panic!("Expected binary expression"),
                                        }
                                    }
                                    _ => panic!("Expected expression statement"),
                                }
                            }
                            _ => panic!("Expected anon function"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
                match &stmts[1] {
                    Statement::Expr(expr) => {
                        match expr {
                            Expr::Call(call) => {
                                match call.callee.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "f"),
                                    _ => panic!("Expected variable f"),
                                }
                                assert_eq!(call.args.len(), 1);
                                match &call.args[0] {
                                    Expr::Literal(Literal::Integer(5)) => {}
                                    _ => panic!("Expected integer 5"),
                                }
                            }
                            _ => panic!("Expected call"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_method_call_with_colon() {
        let src = r#"
            fun test()
                Test:foo()
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Expr(expr) => {
                        match expr {
                            Expr::MethodCallColon(mc) => {
                                assert_eq!(mc.class, "test/Test");
                                assert_eq!(mc.method, "foo");
                                assert!(mc.args.is_empty());
                            }
                            _ => panic!("Expected MethodCallColon"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_method_call_with_dot() {
        let src = r#"
            fun test()
                obj.foo()
                obj.bar(1, 2)
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmts = &func.body.statements;
                assert_eq!(stmts.len(), 2);

                match &stmts[0] {
                    Statement::Expr(expr) => {
                        match expr {
                            Expr::MethodCallDot(mc) => {
                                match mc.target.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "obj"),
                                    _ => panic!("Expected variable obj"),
                                }
                                assert_eq!(mc.method, "foo");
                                assert!(mc.args.is_empty());
                            }
                            _ => panic!("Expected MethodCallDot for obj.foo()"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }

                match &stmts[1] {
                    Statement::Expr(expr) => {
                        match expr {
                            Expr::MethodCallDot(mc) => {
                                match mc.target.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "obj"),
                                    _ => panic!("Expected variable obj"),
                                }
                                assert_eq!(mc.method, "bar");
                                assert_eq!(mc.args.len(), 2);
                                match &mc.args[0] {
                                    Expr::Literal(Literal::Integer(1)) => {}
                                    _ => panic!("Expected 1"),
                                }
                                match &mc.args[1] {
                                    Expr::Literal(Literal::Integer(2)) => {}
                                    _ => panic!("Expected 2"),
                                }
                            }
                            _ => panic!("Expected MethodCallDot for obj.bar()"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_chained_method_calls() {
        let src = r#"
            fun test()
                obj.foo().bar(3)
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Expr(expr) => {
                        match expr {
                            Expr::MethodCallDot(mc) => {
                                assert_eq!(mc.method, "bar");
                                assert_eq!(mc.args.len(), 1);
                                match &mc.args[0] {
                                    Expr::Literal(Literal::Integer(3)) => {}
                                    _ => panic!("Expected 3"),
                                }
                                match mc.target.as_ref() {
                                    Expr::MethodCallDot(inner) => {
                                        match inner.target.as_ref() {
                                            Expr::Var(name) => assert_eq!(name, "obj"),
                                            _ => panic!("Expected obj"),
                                        }
                                        assert_eq!(inner.method, "foo");
                                        assert!(inner.args.is_empty());
                                    }
                                    _ => panic!("Expected nested MethodCallDot"),
                                }
                            }
                            _ => panic!("Expected MethodCallDot"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_assignment_plus() {
        let src = r#"
            fun test()
                let x = 10
                x =+ 5
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmts = &func.body.statements;
                assert_eq!(stmts.len(), 2);
                match &stmts[1] {
                    Statement::Assign(assign) => {
                        match &assign.target {
                            AssignTarget::Var(name) => assert_eq!(name, "x"),
                            _ => panic!("Expected variable target"),
                        }
                        assert!(matches!(assign.op, AssignOp::PlusEq));
                        match &assign.value {
                            Expr::Literal(Literal::Integer(5)) => {}
                            _ => panic!("Expected integer 5"),
                        }
                    }
                    _ => panic!("Expected Assign statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_array_literal_and_index_assignment() {
        let src = r#"
            fun test_array()
                let arr = [10, 20, 30]
                arr[0] = 100
                arr[1] =+ 5
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmts = &func.body.statements;
                assert_eq!(stmts.len(), 3);
                match &stmts[0] {
                    Statement::Let(let_stmt) => {
                        assert_eq!(let_stmt.name, "arr");
                        match &let_stmt.value {
                            Expr::Literal(Literal::Array(elements)) => {
                                assert_eq!(elements.len(), 3);
                                match &elements[0] {
                                    Expr::Literal(Literal::Integer(10)) => {}
                                    _ => panic!("Expected 10"),
                                }
                                match &elements[1] {
                                    Expr::Literal(Literal::Integer(20)) => {}
                                    _ => panic!("Expected 20"),
                                }
                                match &elements[2] {
                                    Expr::Literal(Literal::Integer(30)) => {}
                                    _ => panic!("Expected 30"),
                                }
                            }
                            _ => panic!("Expected array literal"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
                match &stmts[1] {
                    Statement::Assign(assign) => {
                        match &assign.target {
                            AssignTarget::Index { target, index } => {
                                match target.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "arr"),
                                    _ => panic!("Expected variable arr"),
                                }
                                match index.as_ref() {
                                    Expr::Literal(Literal::Integer(0)) => {}
                                    _ => panic!("Expected index 0"),
                                }
                            }
                            _ => panic!("Expected index assignment"),
                        }
                        assert!(matches!(assign.op, AssignOp::Assign));
                        match &assign.value {
                            Expr::Literal(Literal::Integer(100)) => {}
                            _ => panic!("Expected 100"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
                match &stmts[2] {
                    Statement::Assign(assign) => {
                        match &assign.target {
                            AssignTarget::Index { target, index } => {
                                match target.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "arr"),
                                    _ => panic!("Expected variable arr"),
                                }
                                match index.as_ref() {
                                    Expr::Literal(Literal::Integer(1)) => {}
                                    _ => panic!("Expected index 1"),
                                }
                            }
                            _ => panic!("Expected index assignment"),
                        }
                        assert!(matches!(assign.op, AssignOp::PlusEq));
                        match &assign.value {
                            Expr::Literal(Literal::Integer(5)) => {}
                            _ => panic!("Expected 5"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_map_literal_and_index_assignment() {
        let src = r#"
            fun test_map()
                let map = { "key": 123 }
                map["key"] = 456
                map["other"] =+ 789
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmts = &func.body.statements;
                assert_eq!(stmts.len(), 3);
                match &stmts[0] {
                    Statement::Let(let_stmt) => {
                        assert_eq!(let_stmt.name, "map");
                        match &let_stmt.value {
                            Expr::Literal(Literal::Map(pairs)) => {
                                assert_eq!(pairs.len(), 1);
                                let (key, value) = &pairs[0];
                                match key {
                                    Expr::Literal(Literal::String(s)) => assert_eq!(s, "key"),
                                    _ => panic!("Expected string key"),
                                }
                                match value {
                                    Expr::Literal(Literal::Integer(123)) => {}
                                    _ => panic!("Expected 123"),
                                }
                            }
                            _ => panic!("Expected map literal"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
                match &stmts[1] {
                    Statement::Assign(assign) => {
                        match &assign.target {
                            AssignTarget::Index { target, index } => {
                                match target.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "map"),
                                    _ => panic!("Expected variable map"),
                                }
                                match index.as_ref() {
                                    Expr::Literal(Literal::String(s)) => assert_eq!(s, "key"),
                                    _ => panic!("Expected string index"),
                                }
                            }
                            _ => panic!("Expected index assignment"),
                        }
                        assert!(matches!(assign.op, AssignOp::Assign));
                        match &assign.value {
                            Expr::Literal(Literal::Integer(456)) => {}
                            _ => panic!("Expected 456"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
                match &stmts[2] {
                    Statement::Assign(assign) => {
                        match &assign.target {
                            AssignTarget::Index { target, index } => {
                                match target.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "map"),
                                    _ => panic!("Expected variable map"),
                                }
                                match index.as_ref() {
                                    Expr::Literal(Literal::String(s)) => assert_eq!(s, "other"),
                                    _ => panic!("Expected string index"),
                                }
                            }
                            _ => panic!("Expected index assignment"),
                        }
                        assert!(matches!(assign.op, AssignOp::PlusEq));
                        match &assign.value {
                            Expr::Literal(Literal::Integer(789)) => {}
                            _ => panic!("Expected 789"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_return_statement() {
        let src = r#"
            fun test_return()
                return 42
            end
            fun test_return_void()
                return
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        assert_eq!(module.items.len(), 2);
        match &module.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Return(Some(expr)) => {
                        match expr {
                            Expr::Literal(Literal::Integer(42)) => {}
                            _ => panic!("Expected 42"),
                        }
                    }
                    _ => panic!("Expected Return with value"),
                }
            }
            _ => panic!("Expected Function"),
        }
        match &module.items[1] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Return(None) => {}
                    _ => panic!("Expected Return without value"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_nested_blocks() {
        let src = r#"
            fun test_blocks()
                block
                    let a = 1
                end
                block
                    let b = 2
                end
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 2);
                match &func.body.statements[0] {
                    Statement::Block(block) => {
                        assert_eq!(block.statements.len(), 1);
                        match &block.statements[0] {
                            Statement::Let(let_stmt) => {
                                assert_eq!(let_stmt.name, "a");
                                match &let_stmt.value {
                                    Expr::Literal(Literal::Integer(1)) => {}
                                    _ => panic!("Expected 1"),
                                }
                            }
                            _ => panic!("Expected Let"),
                        }
                    }
                    _ => panic!("Expected Block"),
                }
                match &func.body.statements[1] {
                    Statement::Block(block) => {
                        assert_eq!(block.statements.len(), 1);
                        match &block.statements[0] {
                            Statement::Let(let_stmt) => {
                                assert_eq!(let_stmt.name, "b");
                                match &let_stmt.value {
                                    Expr::Literal(Literal::Integer(2)) => {}
                                    _ => panic!("Expected 2"),
                                }
                            }
                            _ => panic!("Expected Let"),
                        }
                    }
                    _ => panic!("Expected Block"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_multiple_inheritance() {
        let src = r#"
            class Foo extends Base1 and Base2 and Base3
                def method()
                    let x = 1
                end
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Class(cls) => {
                assert_eq!(cls.name, "Foo");
                assert_eq!(cls.extends, vec!["test/Base1", "test/Base2", "test/Base3"]);
                assert_eq!(cls.methods.len(), 1);
            }
            _ => panic!("Expected Class"),
        }
    }

    #[test]
    fn test_closure_with_captures() {
        let src = r#"
            fun outer()
                let a = 10
                let f = fun [a] (x) a + x end
                f(5)
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmts = &func.body.statements;
                assert_eq!(stmts.len(), 3);
                match &stmts[1] {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::Function(anon) => {
                                assert_eq!(anon.captures, Some(vec!["a".to_string()]));
                                assert_eq!(anon.params, vec!["x"]);
                                assert_eq!(anon.body.statements.len(), 1);
                                match &anon.body.statements[0] {
                                    Statement::Expr(expr) => {
                                        match expr {
                                            Expr::Binary(bin) => {
                                                assert!(matches!(bin.op, BinaryOp::Add));
                                                match bin.left.as_ref() {
                                                    Expr::Var(name) => assert_eq!(name, "a"),
                                                    _ => panic!("Expected captured variable a"),
                                                }
                                                match bin.right.as_ref() {
                                                    Expr::Var(name) => assert_eq!(name, "x"),
                                                    _ => panic!("Expected parameter x"),
                                                }
                                            }
                                            _ => panic!("Expected binary expression"),
                                        }
                                    }
                                    _ => panic!("Expected expression statement"),
                                }
                            }
                            _ => panic!("Expected anon function"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_closure_with_star_capture() {
        let src = r#"
            fun outer()
                let f = fun [*] (x) x + 1 end
                f(5)
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmts = &func.body.statements;
                assert_eq!(stmts.len(), 2);
                match &stmts[0] {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::Function(anon) => {
                                assert_eq!(anon.captures, None);
                                assert_eq!(anon.params, vec!["x"]);
                                assert_eq!(anon.body.statements.len(), 1);
                                match &anon.body.statements[0] {
                                    Statement::Expr(expr) => {
                                        match expr {
                                            Expr::Binary(bin) => {
                                                assert!(matches!(bin.op, BinaryOp::Add));
                                                match bin.left.as_ref() {
                                                    Expr::Var(name) => assert_eq!(name, "x"),
                                                    _ => panic!("Expected parameter x"),
                                                }
                                                match bin.right.as_ref() {
                                                    Expr::Literal(Literal::Integer(1)) => {}
                                                    _ => panic!("Expected 1"),
                                                }
                                            }
                                            _ => panic!("Expected binary expression"),
                                        }
                                    }
                                    _ => panic!("Expected expression statement"),
                                }
                            }
                            _ => panic!("Expected anon function"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_member_assignment() {
        let src = r#"
            class MyClass
                def init()
                    this.field = 100
                    this.field =+ 50
                end
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Class(cls) => {
                assert_eq!(cls.name, "MyClass");
                let method = &cls.methods[0];
                assert_eq!(method.name, "init");
                let stmts = &method.body.statements;
                assert_eq!(stmts.len(), 2);
                match &stmts[0] {
                    Statement::Assign(assign) => {
                        match &assign.target {
                            AssignTarget::Member { target, name } => {
                                match target.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "this"),
                                    _ => panic!("Expected this"),
                                }
                                assert_eq!(name, "field");
                            }
                            _ => panic!("Expected member assignment"),
                        }
                        assert!(matches!(assign.op, AssignOp::Assign));
                        match &assign.value {
                            Expr::Literal(Literal::Integer(100)) => {}
                            _ => panic!("Expected 100"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
                match &stmts[1] {
                    Statement::Assign(assign) => {
                        match &assign.target {
                            AssignTarget::Member { target, name } => {
                                match target.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "this"),
                                    _ => panic!("Expected this"),
                                }
                                assert_eq!(name, "field");
                            }
                            _ => panic!("Expected member assignment"),
                        }
                        assert!(matches!(assign.op, AssignOp::PlusEq));
                        match &assign.value {
                            Expr::Literal(Literal::Integer(50)) => {}
                            _ => panic!("Expected 50"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }
            }
            _ => panic!("Expected Class"),
        }
    }

    #[test]
    fn test_new_with_arguments() {
        let src = r#"
            fun test_new()
                let obj = new MyClass(10, "hello")
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::New(new_expr) => {
                                assert_eq!(new_expr.class, "test/MyClass");
                                assert_eq!(new_expr.args.len(), 2);
                                match &new_expr.args[0] {
                                    Expr::Literal(Literal::Integer(10)) => {}
                                    _ => panic!("Expected 10"),
                                }
                                match &new_expr.args[1] {
                                    Expr::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
                                    _ => panic!("Expected 'hello'"),
                                }
                            }
                            _ => panic!("Expected New"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_complex_expression_precedence() {
        let src = r#"
            fun test_prec()
                let a = 1 + 2 * 3 - 4 / 2 + 5
                let b = 1 + 2 * (3 - 4) / 2
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        assert_eq!(module.items.len(), 1);
        match &module.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 2);
                match &func.body.statements[0] {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::Binary(_) => {}
                            _ => panic!("Expected binary expression for first"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
                match &func.body.statements[1] {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::Binary(_) => {}
                            _ => panic!("Expected binary expression for second"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_function_with_parameters() {
        let src = r#"
            fun add(x, y)
                return x + y
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                assert_eq!(func.name, "add");
                assert_eq!(func.params, vec!["x", "y"]);
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Return(Some(expr)) => {
                        match expr {
                            Expr::Binary(bin) => {
                                assert!(matches!(bin.op, BinaryOp::Add));
                                match bin.left.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "x"),
                                    _ => panic!("Expected x"),
                                }
                                match bin.right.as_ref() {
                                    Expr::Var(name) => assert_eq!(name, "y"),
                                    _ => panic!("Expected y"),
                                }
                            }
                            _ => panic!("Expected binary expression"),
                        }
                    }
                    _ => panic!("Expected Return"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_try_catch_throw() {
        let src = r#"
            use io
            fun test()
                try
                    throw "error"
                catch(e)
                    io::println(e)
                end
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["io"]);
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::TryCatch(tc) => {
                        assert_eq!(tc.try_block.statements.len(), 1);
                        match &tc.try_block.statements[0] {
                            Statement::Throw(expr) => {
                                match expr {
                                    Expr::Literal(Literal::String(s)) => assert_eq!(s, "error"),
                                    _ => panic!("Expected string literal"),
                                }
                            }
                            _ => panic!("Expected throw"),
                        }
                        assert_eq!(tc.catch_param, "e");
                        assert_eq!(tc.catch_block.statements.len(), 1);
                    }
                    _ => panic!("Expected TryCatch"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    // ---- Тесты разрешения use ----

    #[test]
    fn test_use_resolves_class_name() {
        let src = r#"
            use std/io as io
            class Foo extends io::Reader
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["std/io"]);
        match &module.items[0] {
            Item::Class(cls) => {
                assert_eq!(cls.extends, vec!["std/io/Reader"]);
                assert_eq!(cls.name, "Foo");
            }
            _ => panic!("Expected Class"),
        }
    }

    #[test]
    fn test_use_alias_resolves() {
        let src = r#"
            use std/io as myio
            class Foo extends myio::Reader
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["std/io"]);
        match &module.items[0] {
            Item::Class(cls) => {
                assert_eq!(cls.extends, vec!["std/io/Reader"]);
                assert_eq!(cls.name, "Foo");
            }
            _ => panic!("Expected Class"),
        }
    }

    #[test]
    fn test_use_without_alias() {
        let src = r#"
            use std/io
            fun test()
                let obj = new io::Writer()
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["std/io"]);
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::New(new_expr) => {
                                assert_eq!(new_expr.class, "std/io/Writer");
                            }
                            _ => panic!("Expected New"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_use_alias_on_simple_name() {
        let src = r#"
            use std/io
            class Foo extends Reader
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["std/io"]);
        match &module.items[0] {
            Item::Class(cls) => {
                assert_eq!(cls.extends, vec!["test/Reader"]);
            }
            _ => panic!("Expected Class"),
        }
    }

    #[test]
    fn test_use_alias_with_module_prefix() {
        let src = r#"
            use std/io as myio
            class Foo extends myio
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["std/io"]);
        match &module.items[0] {
            Item::Class(cls) => {
                assert_eq!(cls.extends, vec!["std/io"]);
            }
            _ => panic!("Expected Class"),
        }
    }

    #[test]
    fn test_method_call_colon_with_qualified_class() {
        let src = r#"
            use std/io as myio
            fun test()
                myio::Reader:method()
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["std/io"]);
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Expr(expr) => {
                        match expr {
                            Expr::MethodCallColon(mc) => {
                                assert_eq!(mc.class, "std/io/Reader");
                                assert_eq!(mc.method, "method");
                                assert!(mc.args.is_empty());
                            }
                            _ => panic!("Expected MethodCallColon"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_module_call_with_alias() {
        let src = r#"
            use std/io as myio
            fun test()
                myio::println("Hello")
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["std/io"]);
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Expr(expr) => {
                        match expr {
                            Expr::ModuleCall(mc) => {
                                assert_eq!(mc.module, "std/io");
                                assert_eq!(mc.function, "println");
                                assert_eq!(mc.args.len(), 1);
                                match &mc.args[0] {
                                    Expr::Literal(Literal::String(s)) => assert_eq!(s, "Hello"),
                                    _ => panic!("Expected string literal"),
                                }
                            }
                            _ => panic!("Expected ModuleCall"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_module_call_with_qualified_path() {
        let src = r#"
            use std
            fun test()
                std::io::println("World")
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["std"]);
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Expr(expr) => {
                        match expr {
                            Expr::ModuleCall(mc) => {
                                assert_eq!(mc.module, "std/io");
                                assert_eq!(mc.function, "println");
                                assert_eq!(mc.args.len(), 1);
                                match &mc.args[0] {
                                    Expr::Literal(Literal::String(s)) => assert_eq!(s, "World"),
                                    _ => panic!("Expected string literal"),
                                }
                            }
                            _ => panic!("Expected ModuleCall"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_unknown_alias_error() {
        let src = r#"
            class Foo extends unknown::Bar
            end
        "#;
        let lexer = Lexer::new("full.yg".to_string(), format!("module test\n{}\nend", src));
        let mut parser = Parser::new(lexer);
        let result = parser.parse_module();
        assert!(result.is_err());
        match result.err().unwrap() {
            ParserError::UnknownAlias(_, _, _, alias) => {
                assert_eq!(alias, "unknown");
            }
            e => panic!("Expected UnknownAlias, got {:?}", e),
        }
    }

    #[test]
    fn test_module_call_unknown_alias() {
        let src = r#"
            fun test()
                unknown::func()
            end
        "#;
        let lexer = Lexer::new("full.yg".to_string(), format!("module test\n{}\nend", src));
        let mut parser = Parser::new(lexer);
        let result = parser.parse_module();
        assert!(result.is_err());
        match result.err().unwrap() {
            ParserError::UnknownAlias(_, _, _, alias) => {
                assert_eq!(alias, "unknown");
            }
            e => panic!("Expected UnknownAlias, got {:?}", e),
        }
    }

    // ---- Тесты для ссылок &X ----

    #[test]
    fn test_object_ref_simple() {
        let src = r#"
            fun test()
                let obj = &MyClass
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::ObjectRef(path) => {
                                assert_eq!(path, "test/MyClass");
                            }
                            _ => panic!("Expected ObjectRef"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_object_ref_with_use() {
        let src = r#"
            use std/io as myio
            fun test()
                let obj = &myio::Reader
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, vec!["std/io"]);
        match &module.items[0] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::ObjectRef(path) => {
                                assert_eq!(path, "std/io/Reader");
                            }
                            _ => panic!("Expected ObjectRef"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    // Проверяем, что &X:F теперь вызывает ошибку
    #[test]
    fn test_function_ref_is_error() {
        let src = r#"
            fun test()
                let f = &MyClass:foo
            end
        "#;
        let lexer = Lexer::new("full.yg".to_string(), format!("module test\n{}\nend", src));
        let mut parser = Parser::new(lexer);
        let result = parser.parse_module();
        assert!(result.is_err());
        match result.err().unwrap() {
            ParserError::UnexpectedTokenRaw(_, _, _, TokenData::Colon) => {}
            e => panic!("Expected UnexpectedTokenRaw with Colon, got {:?}", e),
        }
    }

    // ---- Тесты для this::function ----

    #[test]
    fn test_this_module_call() {
        let src = r#"
            fun foo()
                return 42
            end
            fun test()
                let x = this::foo()
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        assert_eq!(module.items.len(), 2);
        match &module.items[1] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::ModuleCall(mc) => {
                                assert_eq!(mc.module, "test");
                                assert_eq!(mc.function, "foo");
                                assert!(mc.args.is_empty());
                            }
                            _ => panic!("Expected ModuleCall from this::"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_this_module_call_with_nested_path() {
        let src = r#"
            fun foo()
                return 42
            end
            fun test()
                let x = this::bar::foo()
            end
        "#;
        let module = parse_test(src);
        assert_eq!(module.uses, Vec::<String>::new());
        match &module.items[1] {
            Item::Function(func) => {
                let stmt = &func.body.statements[0];
                match stmt {
                    Statement::Let(let_stmt) => {
                        match &let_stmt.value {
                            Expr::ModuleCall(mc) => {
                                assert_eq!(mc.module, "test/bar");
                                assert_eq!(mc.function, "foo");
                                assert!(mc.args.is_empty());
                            }
                            _ => panic!("Expected ModuleCall from this::bar::foo"),
                        }
                    }
                    _ => panic!("Expected Let"),
                }
            }
            _ => panic!("Expected Function"),
        }
    }
}