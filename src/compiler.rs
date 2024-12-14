use crate::{
    chunk::Chunk,
    debug::disassemble_chunk,
    opcode::OpCode,
    scanner::Scanner,
    token::{Token, TokenType}, value::ValueType,
};
use std::u8;

pub struct Compiler<'scanner, 'chunk> {
    scanner: Scanner<'scanner>,
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
    panic_mode: bool,
    chunk: &'chunk mut Chunk,
    debug: bool
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

    // pub fn compile(&mut self) {
    //     let mut line: usize = 0;
    //     loop {
    //         let token: Token = self.scanner.scan_token();
    //         if token.line != line {
    //             print!("{:4} ", token.line);
    //             line = token.line;
    //         } else {
    //             print!("   | ");
    //         }
    //         println!("{:?} '{}'", token.token_type, token.lexeme);

    //         if token.token_type == TokenType::EOF {
    //             break;
    //         }
    //     }
    // }

    pub fn compile(&mut self) -> bool {
        self.advance();
        self.expression();
        self.consume(TokenType::EOF, "Expect end of expression.");
        self.end_compiler();
        
        !self.had_error
    }

    fn advance(&mut self) {
        self.previous = self.current.take();
        //println!(
        //    "Advance Fn - Prev {:?}, Curr {:?}",
        //    self.previous, self.current
        //);

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
                _ => return,
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
                TokenType::PLUS => self.emit_byte(OpCode::ADD as u8),
                TokenType::MINUS => self.emit_byte(OpCode::SUBTRACT as u8),
                TokenType::STAR => self.emit_byte(OpCode::MULTIPLY as u8),
                TokenType::SLASH => self.emit_byte(OpCode::DIVIDE as u8),
                _ => unreachable!(),
            }
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        //println!("PrsPre Fn - Prev: {:?}, Curr: {:?}", self.previous, self.current);
        if let Some(prev_token) = self.previous.as_ref() {
            match Self::get_rule(prev_token.token_type.clone()).prefix {
                Some(prefix_rule) => {
                    prefix_rule(self);
                    while precedence as u8 <= Self::get_rule(self.current.as_ref().unwrap().clone().token_type).precedence as u8 {
                        self.advance();
                        if let Some(infix_rule) = Self::get_rule(self.previous.as_ref().unwrap().clone().token_type).infix {
                            infix_rule(self);
                        }
                    }
                },
                None => {
                    self.error("Expect expression");
                    return;
                }
            }
        }
    }

    fn emit_constant(&mut self, value: ValueType) {
        let constant_byte = self.make_constant(value);
        self.emit_bytes(OpCode::CONSTANT as u8, constant_byte);
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
        if self.debug {
            if !self.had_error {
                disassemble_chunk(&self.chunk, "code");
            }
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::RETURN as u8);
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
            TokenType::RIGHTPAREN => ParseRule::new(None, None, Precedence::NONE),
            TokenType::LEFTBRACE => ParseRule::new(None, None, Precedence::NONE),
            TokenType::RIGHTBRACE => ParseRule::new(None, None, Precedence::NONE),
            TokenType::COMMA => ParseRule::new(None, None, Precedence::NONE),
            TokenType::DOT => ParseRule::new(None, None, Precedence::NONE),
            TokenType::MINUS => ParseRule::new(
                Some(Self::parse_unary),
                Some(Self::parse_binary),
                Precedence::TERM,
            ),
            TokenType::PLUS => ParseRule::new(None, Some(Self::parse_binary), Precedence::TERM),
            TokenType::SEMICOLON => ParseRule::new(None, None, Precedence::NONE),
            TokenType::SLASH => ParseRule::new(None, Some(Self::parse_binary), Precedence::FACTOR),
            TokenType::STAR => ParseRule::new(None, Some(Self::parse_binary), Precedence::FACTOR),
            TokenType::BANG => ParseRule::new(None, None, Precedence::NONE),
            TokenType::BANGEQUAL => ParseRule::new(None, None, Precedence::NONE),
            TokenType::EQUAL => ParseRule::new(None, None, Precedence::NONE),
            TokenType::EQUALEQUAL => ParseRule::new(None, None, Precedence::NONE),
            TokenType::GREATER => ParseRule::new(None, None, Precedence::NONE),
            TokenType::GREATEREQUAL => ParseRule::new(None, None, Precedence::NONE),
            TokenType::LESS => ParseRule::new(None, None, Precedence::NONE),
            TokenType::LESSEQUAL => ParseRule::new(None, None, Precedence::NONE),
            TokenType::IDENTIFIER => ParseRule::new(None, None, Precedence::NONE),
            TokenType::STRING => ParseRule::new(None, None, Precedence::NONE),
            TokenType::NUMBER => ParseRule::new(Some(Self::parse_number), None, Precedence::NONE),
            TokenType::AND => ParseRule::new(None, None, Precedence::NONE),
            TokenType::CLASS => ParseRule::new(None, None, Precedence::NONE),
            TokenType::ELSE => ParseRule::new(None, None, Precedence::NONE),
            TokenType::FALSE => ParseRule::new(None, None, Precedence::NONE),
            TokenType::FOR => ParseRule::new(None, None, Precedence::NONE),
            TokenType::FUN => ParseRule::new(None, None, Precedence::NONE),
            TokenType::IF => ParseRule::new(None, None, Precedence::NONE),
            TokenType::NIL => ParseRule::new(None, None, Precedence::NONE),
            TokenType::OR => ParseRule::new(None, None, Precedence::NONE),
            TokenType::PRINT => ParseRule::new(None, None, Precedence::NONE),
            TokenType::RETURN => ParseRule::new(None, None, Precedence::NONE),
            TokenType::SUPER => ParseRule::new(None, None, Precedence::NONE),
            TokenType::THIS => ParseRule::new(None, None, Precedence::NONE),
            TokenType::TRUE => ParseRule::new(None, None, Precedence::NONE),
            TokenType::VAR => ParseRule::new(None, None, Precedence::NONE),
            TokenType::WHILE => ParseRule::new(None, None, Precedence::NONE),
            TokenType::ERROR => ParseRule::new(None, None, Precedence::NONE),
            TokenType::EOF => ParseRule::new(None, None, Precedence::NONE),
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
