#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub length: usize,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, start: usize, length: usize, line: usize) -> Self {
        Token {
            token_type,
            start,
            length,
            line,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Scanner<'a> {
    line: usize,
    start: usize,
    current: usize,
    source: &'a [u8],
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            line: 1,
            start: 0,
            current: 0,
            source: source.as_bytes(),
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let character = self.peek();
            match character {
                ' ' | '\r' | '\t' => {
                    self.advance();
                    break;
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                    break;
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                    break;
                }
                _ => return,
            }
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        let character = self.advance();

        match character {
            '(' => return self.make_token(TokenType::LEFT_PAREN),
            ')' => return self.make_token(TokenType::RIGHT_PAREN),
            '{' => return self.make_token(TokenType::LEFT_BRACE),
            '}' => return self.make_token(TokenType::RIGHT_BRACE),
            ';' => return self.make_token(TokenType::SEMICOLON),
            ',' => return self.make_token(TokenType::COMMA),
            '.' => return self.make_token(TokenType::DOT),
            '-' => return self.make_token(TokenType::MINUS),
            '+' => return self.make_token(TokenType::PLUS),
            '/' => return self.make_token(TokenType::SLASH),
            '*' => return self.make_token(TokenType::STAR),
            '!' => {
                if self.match_token('=') {
                    return self.make_token(TokenType::BANG_EQUAL);
                }
                return self.make_token(TokenType::BANG);
            }
            '=' => {
                if self.match_token('=') {
                    return self.make_token(TokenType::EQUAL_EQUAL);
                }
                return self.make_token(TokenType::EQUAL);
            }
            '<' => {
                if self.match_token('=') {
                    return self.make_token(TokenType::LESS_EQUAL);
                }
                return self.make_token(TokenType::LESS);
            }
            '>' => {
                if self.match_token('=') {
                    return self.make_token(TokenType::GREATER_EQUAL);
                }
                return self.make_token(TokenType::GREATER_EQUAL);
            }
            '"' => return self.match_string(),
            '0'..='9' => self.match_number(),
            'a'..='z' | 'A'..='Z' | '_' => {
                while self.peek().is_ascii_alphabetic()
                    || self.peek().is_ascii_digit()
                    || self.peek() == '_'
                {
                    self.advance();
                }
                return self.identifier_type();
            }
            _ => return self.error_token("Unexpected character."),
        }
    }

    fn identifier_type(&mut self) -> Token {
        match self.source[self.start] as char {
            'a' => self.check_keyword(1, 2, "nd", TokenType::AND),
            'c' => self.check_keyword(1, 4, "lass", TokenType::CLASS),
            'e' => self.check_keyword(1, 3, "lse", TokenType::ELSE),
            'f' => {
                if self.current - self.start > 1 {
                    match self.source[self.start + 1] as char {
                        'a' => self.check_keyword(2, 3, "lse", TokenType::FALSE),
                        'o' => self.check_keyword(2, 1, "r", TokenType::FOR),
                        'u' => self.check_keyword(2, 1, "n", TokenType::FUN),
                        _ => self.make_token(TokenType::IDENTIFIER)
                    }
                } else {
                    self.make_token(TokenType::IDENTIFIER)
                }
            }
            'i' => self.check_keyword(1, 1, "f", TokenType::IF),
            'n' => self.check_keyword(1, 2, "il", TokenType::NIL),
            'o' => self.check_keyword(1, 1, "r", TokenType::OR),
            'p' => self.check_keyword(1, 4, "rint", TokenType::PRINT),
            'r' => self.check_keyword(1, 5, "eturn", TokenType::RETURN),
            's' => self.check_keyword(1, 4, "uper", TokenType::SUPER),
            't' => {
                if self.current - self.start > 1 {
                    match self.source[self.start + 1] as char {
                        'h' => self.check_keyword(2, 2, "is", TokenType::THIS),
                        'r' => self.check_keyword(2, 2, "ue", TokenType::TRUE),
                        _ => self.make_token(TokenType::IDENTIFIER)
                    }
                } else {
                    self.make_token(TokenType::IDENTIFIER)
                }
            }
            'v' => self.check_keyword(1, 2, "ar", TokenType::VAR),
            'w' => self.check_keyword(1, 4, "hile", TokenType::WHILE),
            _ => self.make_token(TokenType::IDENTIFIER)
        }
    }

    fn check_keyword(&mut self, start: usize, length: usize, rest: &str, token_type: TokenType) -> Token {
        let slice = &self.source[self.start + start..self.start + start + length];
        if let Ok(slice_str) = std::str::from_utf8(slice) {
            if self.current - self.start == start + length && slice_str == rest {
                return self.make_token(token_type);
            }
        }
        self.make_token(TokenType::IDENTIFIER)
    }

    fn match_number(&mut self) -> Token {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::NUMBER)
    }

    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn match_string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string");
        }

        self.advance();
        self.make_token(TokenType::STRING)
    }

    fn match_token(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] as char != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current + 1].into()
    }

    fn peek(&self) -> char {
        self.source[self.current].into()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1].into()
    }

    fn make_token(&mut self, token_type: TokenType) -> Token {
        Token::new(token_type, self.start, self.current - self.start, self.line)
    }

    fn error_token(&mut self, err_msg: &str) -> Token {
        Token::new(
            TokenType::ERROR(err_msg.to_string()),
            self.start,
            err_msg.len(),
            self.line,
        )
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,
    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,
    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,
    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FOR,
    FUN,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    ERROR(String),
    EOF,
}
