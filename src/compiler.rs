use std::u8;

use crate::{chunk::{Chunk, OpCode}, scanner::{self, Scanner, Token, TokenType}};

pub struct Parser {
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
}

impl Parser {
    fn new() -> Self {
        Parser { current: None, previous: None, had_error: false }
    }
}

pub struct Compiler<'a, 'b> {
    scanner: Scanner<'a>,
    parser: Parser,
    chunk: &'b mut Chunk
}

impl<'a, 'b> Compiler<'a, 'b> {
    pub fn new(source: &'a str, chunk: &'b mut Chunk) -> Self {
        Compiler {
            scanner: Scanner::new(source),
            parser: Parser::new(),
            chunk
        }
    }

    fn parse_number(&mut self) {
        if let Some(prev_token) = &self.parser.previous {
            let start = prev_token.start;
            let length = prev_token.length;

            if let Some(constant) = self.scanner.get_slice_constant(start, start + length) {
                let constant_byte = self.chunk.add_constant(constant) as u8;
                self.emit_bytes(OpCode::CONSTANT as u8, constant_byte);
            }
        }
    }

    fn parse_grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
    }

    fn parse_unary(&mut self) {
        if let Some(operator_type) = self.parser.previous.clone() {
            self.expression();
            match operator_type.token_type {
                TokenType::MINUS => self.emit_byte(OpCode::NEGATE as u8),
                _ => return
            }
        }
    }

    fn make_constant(&mut self, value: f64) -> u8 {
        let constant = self.chunk.add_constant(value);

        if constant > 255 {
            if let Some(curr_token) = self.parser.current.clone() {
                self.error_at(&curr_token, "Too many constants in one chunk.");
            }
            return 0;
        }

        constant as u8
    }

    pub fn compile(&mut self) -> bool {
        self.advance();
        self.expression();
        self.consume(TokenType::EOF, "Expect end of expression.");
        self.end_compiler();
        !self.parser.had_error
    }

    pub fn emit_byte(&mut self, byte: u8) {
        if let Some(prev_token) = &self.parser.previous {
            self.chunk.write(byte, prev_token.line);
        }
    }

    pub fn emit_bytes(&mut self, byte_a: u8, byte_b: u8) {
        self.emit_byte(byte_a);
        self.emit_byte(byte_b);
    }

    fn end_compiler(&mut self) {
        self.emit_return()
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::RETURN as u8);
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.take();

        loop {
            self.parser.current = Some(self.scanner.scan_token());
            
            if let Some(curr_token) = self.parser.current.take() {
                match curr_token.token_type {
                    TokenType::ERROR(_) => {
                        self.error_at(&curr_token, "",);
                    }
                    _ => {
                        break;
                    }
                }
            }
        }
    }

    fn expression(&mut self) {
        todo!()
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) {
        if let Some(curr_token) = self.parser.current.clone() {
            if curr_token.token_type == token_type {
                self.advance();
                return;
            }   
            self.error_at(&curr_token, msg); 
        }
    }

    fn error_at(&mut self, token: &Token, msg: &str) {
        eprint!("[line {}] Error ", token.line);

        match &token.token_type {
            TokenType::EOF => eprint!("at end"),
            TokenType::ERROR(s) => eprint!("{s}"),
            _ => eprint!("at {} {}", token.length, token.start),
        }

        println!(": {}", msg);
        self.parser.had_error = true;
    }
}