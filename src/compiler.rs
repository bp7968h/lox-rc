use crate::{
    chunk::Chunk, debug::disassemble_chunk, object::{Object, ObjectType}, opcode::OpCode, scanner::Scanner, token::{Token, TokenType}, value::ValueType
};
use std::default::Default;

pub struct Compiler<'scanner, 'chunk> {
    scanner: Scanner<'scanner>,
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
    panic_mode: bool,
    chunk: &'chunk mut Chunk,
    debug: bool,
}

impl<'scanner, 'chunk> Compiler<'scanner, 'chunk> {
    pub fn new(source: &'scanner str, chunk: &'chunk mut Chunk) -> Self {
        Compiler {
            scanner: Scanner::new(source),
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
            debug: false,
            chunk,
        }
    }

    pub fn compile(&mut self) -> bool {
        self.advance();
        // self.expression();
        // self.consume(TokenType::EOF, "Expect end of expression.");
        while !self.match_token(TokenType::EOF) {
            self.declaration()
        }
        self.end_compiler();

        !self.had_error
    }

    fn declaration(&mut self) {
        self.statement();

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::PRINT) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        if let Some(curr_token) = self.current.take() {
            while curr_token.token_type != TokenType::EOF {
                if let Some(prev_token) = self.previous.as_ref() {
                    if prev_token.token_type == TokenType::SEMICOLON {
                        return;
                    }
                }

                match curr_token.token_type {
                    TokenType::CLASS | TokenType::FUN |
                    TokenType::VAR | TokenType::FOR |
                    TokenType::IF | TokenType::WHILE |
                    TokenType::PRINT | TokenType::RETURN => return,
                    _ => ()
                }

                self.advance();
            }
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.");
        self.emit_byte(OpCode::PRINT.into());
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.");
        self.emit_byte(OpCode::POP.into());
    }

    fn advance(&mut self) {
        self.previous = self.current.take();
        // println!(
        //    "Advance Fn - Prev {:?}, Curr {:?}",
        //    self.previous, self.current
        // );

        loop {
            let scanned_token = self.scanner.scan_token();

            match scanned_token.token_type {
                TokenType::ERROR => {
                    let err_message = scanned_token.lexeme.clone();
                    self.current = Some(scanned_token);
                    self.error_at_current(&err_message);
                }
                _ => {
                    self.current = Some(scanned_token);
                    break;
                }
            }
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::ASSIGNMENT);
    }

    fn parse_number(&mut self) {
        if let Some(prev_token) = self.previous.as_ref() {
            match prev_token.lexeme.parse::<f64>() {
                Ok(num_value) => {
                    // self.emit_constant(num_value);
                    self.emit_constant(ValueType::Number(num_value));
                }
                Err(e) => {
                    let err_str = e.to_string();
                    self.error(&err_str)
                }
            }
        }
    }

    fn parse_grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RIGHTPAREN, "Expect ')' after expression.");
    }

    fn parse_unary(&mut self) {
        if let Some(operator_type) = self.previous.take() {
            self.parse_precedence(Precedence::UNARY);
            match operator_type.token_type {
                TokenType::MINUS => self.emit_byte(OpCode::NEGATE as u8),
                TokenType::BANG => self.emit_byte(OpCode::NOT as u8),
                _ => (),
            }
        }
    }

    fn parse_binary(&mut self) {
        if let Some(operator) = self.previous.take() {
            let operator_type = &operator.token_type;
            let rule = Self::get_rule(*operator_type);
            let next_precedence = Precedence::from(rule.precedence as u8 + 1);
            self.parse_precedence(next_precedence);
            match operator_type {
                TokenType::BANGEQUAL => self.emit_bytes(OpCode::EQUAL as u8, OpCode::NOT as u8),
                TokenType::EQUALEQUAL => self.emit_byte(OpCode::EQUAL as u8),
                TokenType::GREATER => self.emit_byte(OpCode::GREATER as u8),
                TokenType::GREATEREQUAL => self.emit_bytes(OpCode::LESS as u8, OpCode::NOT as u8),
                TokenType::LESS => self.emit_byte(OpCode::LESS as u8),
                TokenType::LESSEQUAL => self.emit_bytes(OpCode::GREATER as u8, OpCode::NOT as u8),
                TokenType::PLUS => self.emit_byte(OpCode::ADD as u8),
                TokenType::MINUS => self.emit_byte(OpCode::SUBTRACT as u8),
                TokenType::STAR => self.emit_byte(OpCode::MULTIPLY as u8),
                TokenType::SLASH => self.emit_byte(OpCode::DIVIDE as u8),
                _ => unreachable!(),
            }
        }
    }

    fn parse_literal(&mut self) {
        if let Some(token) = self.previous.as_ref() {
            match token.token_type {
                TokenType::FALSE => self.emit_byte(OpCode::FALSE as u8),
                TokenType::NIL => self.emit_byte(OpCode::NIL as u8),
                TokenType::TRUE => self.emit_byte(OpCode::TRUE as u8),
                _ => (), // unreachable!()
            }
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        //println!("PrsPre Fn - Prev: {:?}, Curr: {:?}", self.previous, self.current);
        if let Some(prev_token) = self.previous.as_ref() {
            match Self::get_rule(prev_token.token_type).prefix {
                Some(prefix_rule) => {
                    prefix_rule(self);
                    while precedence as u8
                        <= Self::get_rule(self.current.as_ref().unwrap().clone().token_type)
                            .precedence as u8
                    {
                        self.advance();
                        if let Some(infix_rule) =
                            Self::get_rule(self.previous.as_ref().unwrap().clone().token_type).infix
                        {
                            infix_rule(self);
                        }
                    }
                }
                None => {
                    self.error("Expect expression");
                }
            }
        }
    }

    fn parse_string(&mut self) {
        if let Some(prev_token) = self.previous.as_mut() {
            let str_value = std::mem::take(&mut prev_token.lexeme);
            let str_obj = ObjectType::ObjString(str_value);
            self.emit_constant(ValueType::Obj(Object::new(str_obj)));
        }
    }

    fn emit_constant(&mut self, value: ValueType) {
        let cons_byte_idx = self.make_constant(value);
        self.emit_bytes(OpCode::CONSTANT as u8, cons_byte_idx);
    }

    fn make_constant(&mut self, value: ValueType) -> u8 {
        let constant = self.chunk.add_constant(value);

        if constant > 255 {
            self.error("Too many constants in one chunk.");
            return 0;
        }

        constant as u8
    }

    pub fn emit_byte(&mut self, byte: u8) {
        if let Some(prev_token) = self.previous.as_ref() {
            self.chunk.write(byte, prev_token.line);
        }
    }

    pub fn emit_bytes(&mut self, byte_a: u8, byte_b: u8) {
        self.emit_byte(byte_a);
        self.emit_byte(byte_b);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if self.debug && !self.had_error {
            disassemble_chunk(self.chunk, "code");
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::RETURN as u8);
    }

    /// Checks if the current token doesn't matches the returnds false
    /// Otherwise, advances the compiler
    fn match_token(&mut self, expected_token: TokenType) -> bool {
        if !self.check_token(expected_token) {
            return false;
        }
        self.advance();
        true
    }

    /// Check if the current token (token_type) matches
    fn check_token(&mut self, expected_token: TokenType) -> bool {
        if let Some(curr_token) = self.current.as_ref() {
            return curr_token.token_type == expected_token;
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if let Some(curr_token) = self.current.as_ref() {
            if curr_token.token_type == token_type {
                self.advance();
                return;
            }
            self.error_at_current(message);
        }
    }

    fn error_at_current(&mut self, message: &str) {
        if let Some(curr_token) = self.current.take() {
            self.error_at(&curr_token, message);
            self.current = Some(curr_token);
        }
    }

    fn error(&mut self, message: &str) {
        if let Some(prev_token) = self.previous.take() {
            self.error_at(&prev_token, message);
            self.previous = Some(prev_token);
        }
    }

    fn error_at(&mut self, token: &Token, msg: &str) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;
        eprint!("[line {} => Error ", token.line);

        match token.token_type {
            TokenType::EOF => eprint!("at end"),
            TokenType::ERROR => (),
            _ => eprint!("at {}", token.lexeme),
        }

        println!(": {}", msg);
        self.had_error = true;
    }

    fn get_rule(token_type: TokenType) -> ParseRule<'scanner, 'chunk> {
        match token_type {
            TokenType::LEFTPAREN => {
                ParseRule::new(Some(Self::parse_grouping), None, Precedence::NONE)
            }
            TokenType::RIGHTPAREN => ParseRule::default(),
            TokenType::LEFTBRACE => ParseRule::default(),
            TokenType::RIGHTBRACE => ParseRule::default(),
            TokenType::COMMA => ParseRule::default(),
            TokenType::DOT => ParseRule::default(),
            TokenType::MINUS => ParseRule::new(
                Some(Self::parse_unary),
                Some(Self::parse_binary),
                Precedence::TERM,
            ),
            TokenType::PLUS => ParseRule::new(None, Some(Self::parse_binary), Precedence::TERM),
            TokenType::SEMICOLON => ParseRule::default(),
            TokenType::SLASH => ParseRule::new(None, Some(Self::parse_binary), Precedence::FACTOR),
            TokenType::STAR => ParseRule::new(None, Some(Self::parse_binary), Precedence::FACTOR),
            TokenType::BANG => ParseRule::new(Some(Self::parse_unary), None, Precedence::NONE),
            TokenType::BANGEQUAL => {
                ParseRule::new(None, Some(Self::parse_binary), Precedence::EQUALITY)
            }
            TokenType::EQUAL => ParseRule::default(),
            TokenType::EQUALEQUAL => {
                ParseRule::new(None, Some(Self::parse_binary), Precedence::EQUALITY)
            }
            TokenType::GREATER => {
                ParseRule::new(None, Some(Self::parse_binary), Precedence::COMPARISON)
            }
            TokenType::GREATEREQUAL => {
                ParseRule::new(None, Some(Self::parse_binary), Precedence::COMPARISON)
            }
            TokenType::LESS => {
                ParseRule::new(None, Some(Self::parse_binary), Precedence::COMPARISON)
            }
            TokenType::LESSEQUAL => {
                ParseRule::new(None, Some(Self::parse_binary), Precedence::COMPARISON)
            }
            TokenType::IDENTIFIER => ParseRule::default(),
            TokenType::STRING => ParseRule { prefix: Some(Self::parse_string), infix: None, precedence: Precedence::NONE },
            TokenType::NUMBER => ParseRule::new(Some(Self::parse_number), None, Precedence::NONE),
            TokenType::AND => ParseRule::default(),
            TokenType::CLASS => ParseRule::default(),
            TokenType::ELSE => ParseRule::default(),
            TokenType::FALSE => ParseRule::new(Some(Self::parse_literal), None, Precedence::NONE),
            TokenType::FOR => ParseRule::default(),
            TokenType::FUN => ParseRule::default(),
            TokenType::IF => ParseRule::default(),
            TokenType::NIL => ParseRule::new(Some(Self::parse_literal), None, Precedence::NONE),
            TokenType::OR => ParseRule::default(),
            TokenType::PRINT => ParseRule::default(),
            TokenType::RETURN => ParseRule::default(),
            TokenType::SUPER => ParseRule::default(),
            TokenType::THIS => ParseRule::default(),
            TokenType::TRUE => ParseRule::new(Some(Self::parse_literal), None, Precedence::NONE),
            TokenType::VAR => ParseRule::default(),
            TokenType::WHILE => ParseRule::default(),
            TokenType::ERROR => ParseRule::default(),
            TokenType::EOF => ParseRule::default(),
        }
    }
}

type ParseFn<'scanner, 'chunk> = fn(&mut Compiler<'scanner, 'chunk>) -> ();

pub struct ParseRule<'scanner, 'chunk> {
    prefix: Option<ParseFn<'scanner, 'chunk>>,
    infix: Option<ParseFn<'scanner, 'chunk>>,
    precedence: Precedence,
}

impl<'scanner, 'chunk> ParseRule<'scanner, 'chunk> {
    pub fn new(
        prefix: Option<ParseFn<'scanner, 'chunk>>,
        infix: Option<ParseFn<'scanner, 'chunk>>,
        precedence: Precedence,
    ) -> Self {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }
}

impl<'scanner, 'chunk> Default for ParseRule<'scanner, 'chunk> {
    fn default() -> Self {
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Precedence {
    NONE,
    ASSIGNMENT, // =
    OR,         // or
    AND,        // and
    EQUALITY,   // == !=
    COMPARISON, // < > <= >=
    TERM,       // + -
    FACTOR,     // * /
    UNARY,      // ! -
    CALL,       // . ()
    PRIMARY,
}

impl Default for Precedence {
    fn default() -> Self {
        Precedence::NONE
    }
}

impl From<u8> for Precedence {
    fn from(value: u8) -> Self {
        match value {
            1 => Precedence::ASSIGNMENT,
            2 => Precedence::OR,
            3 => Precedence::AND,
            4 => Precedence::EQUALITY,
            5 => Precedence::COMPARISON,
            6 => Precedence::TERM,
            7 => Precedence::FACTOR,
            8 => Precedence::UNARY,
            9 => Precedence::CALL,
            10 => Precedence::PRIMARY,
            _ => Precedence::NONE,
        }
    }
}
