#[derive(Debug, PartialEq)]
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
    source: &'a[u8],
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

    pub fn scan_token(&mut self) -> Token {
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        let character= self.advance();
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

            _ => return self.error_token("Unexpected character.")
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1].into()
    }

    fn make_token(&mut self, token_type: TokenType) -> Token {
        Token::new(
            token_type, 
            self.start, 
            self.current - self.start, 
            self.line
        )
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

#[derive(Debug, PartialEq)]
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
