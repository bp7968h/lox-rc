use crate::{chunk::Chunk, scanner::{self, Scanner, Token, TokenType}};

pub struct Compiler;

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

impl Compiler {
    pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
        let mut scanner = Scanner::new(source);
        let mut parser = Parser::new();
        
        Compiler.advance(&mut scanner, &mut parser);
        Self::expression();
        Self::consume(TokenType::EOF, "Expect end of expression.");
        !parser.had_error
    }

    fn advance(&mut self, scanner: &mut Scanner, parser: &mut Parser) {
        parser.previous = parser.current.take();

        loop {
            parser.current = Some(scanner.scan_token());
            
            if let Some(curr_token) = parser.current.clone() {
                match curr_token.token_type {
                    TokenType::ERROR(_) => {
                        self.error_at(&curr_token, "", parser);
                    }
                    _ => {
                        break;
                    }
                }
            }
        }
    }

    fn expression() {
        todo!()
    }

    fn consume(token_type: TokenType, msg: &str) {
        todo!()
    }

    fn error_at(&self, token: &Token, msg: &str, parser: &mut Parser) {
        eprint!("[line {}] Error ", token.line);

        match &token.token_type {
            TokenType::EOF => eprint!("at end"),
            TokenType::ERROR(s) => eprint!("{s}"),
            _ => eprint!("at {} {}", token.length, token.start),
        }

        println!(": {}", msg);
        parser.had_error = true;
    }
}