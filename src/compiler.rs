use crate::scanner::{Scanner, Token, TokenType};

pub struct Compiler;

impl Compiler {
    pub fn compile(source: &str) {
        let mut scanner = Scanner::new(source);
        let mut line: usize = 0;

        loop {
            let token: Token = scanner.scan_token();
            if token.line != line {
                print!("{:4}", token.line);
                line = token.line;
            } else {
                print!("   | ");
            }
            println!("{:?} '{}:{}'", token.token_type, token.length, token.start);

            if token.token_type == TokenType::EOF {
                break;
            }
        }

    }
}