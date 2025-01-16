use crate::{
    chunk::Chunk,
    debug::disassemble_chunk,
    object::{Object, ObjectType},
    opcode::OpCode,
    scanner::Scanner,
    token::{Token, TokenType},
    value::ValueType,
};
use std::default::Default;

pub struct Compiler<'scanner, 'chunk> {
    scanner: Scanner<'scanner>,
    current: Option<Token>,
    previous: Option<Token>,
    chunk: &'chunk mut Chunk,
    local_track: LocalTracking,
    had_error: bool,
    panic_mode: bool,
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
            local_track: LocalTracking::default(),
            chunk,
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        loop {
            if let Some(curr_token) = self.current.take() {
                if curr_token.token_type != TokenType::EOF {
                    if let Some(prev_token) = self.previous.as_ref() {
                        if prev_token.token_type == TokenType::SEMICOLON {
                            return;
                        }
                    }

                    match curr_token.token_type {
                        TokenType::CLASS
                        | TokenType::FUN
                        | TokenType::VAR
                        | TokenType::FOR
                        | TokenType::IF
                        | TokenType::WHILE
                        | TokenType::PRINT
                        | TokenType::RETURN => break,
                        _ => (),
                    }
                }
            }
            self.advance();
        }
    }

    pub fn compile(&mut self) -> bool {
        self.advance();

        while !self.match_token(TokenType::EOF) {
            self.declaration()
        }
        self.end_compiler();

        !self.had_error
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::VAR) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    /// Parse the Variable and get the index of constant
    /// If the current token is = then next expression value to constant else NIL
    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_token(TokenType::EQUAL) {
            self.expression();
        } else {
            self.emit_byte(OpCode::NIL.into());
        }

        self.consume(
            TokenType::SEMICOLON,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::PRINT) {
            self.print_statement();
        } else if self.match_token(TokenType::IF) {
            self.if_statement();
        } else if self.match_token(TokenType::LEFTBRACE) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.");
        self.emit_byte(OpCode::PRINT.into());
    }

    /// Consume the (, compile the condition expression which leaves condition value on top of stack
    /// Consume the )
    fn if_statement(&mut self) {
        self.consume(TokenType::LEFTPAREN, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RIGHTPAREN, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse as u8);
        self.emit_byte(OpCode::POP as u8);
        self.statement();

        let else_jump = self.emit_jump(OpCode::JUMP as u8);

        self.patch_jump(then_jump);
        self.emit_byte(OpCode::POP as u8);

        if self.match_token(TokenType::ELSE) {
            self.statement();
        }

        self.patch_jump(else_jump);
    }

    fn block(&mut self) {
        while !self.check_token(TokenType::RIGHTBRACE) && !self.check_token(TokenType::EOF) {
            self.declaration();
        }

        self.consume(TokenType::RIGHTBRACE, "Expect '}' after block.");
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

    /// Consumes the Identifier - if the current is `Identifier` then, move forward
    /// Take ownership of the previous token (IDENTIFIER),
    ///     adds the previous token's lexeme to chunk's constant
    /// Return the index of the added constant  
    fn parse_variable(&mut self, err_msg: &str) -> u8 {
        self.consume(TokenType::IDENTIFIER, err_msg);
        self.declare_variable();
        if *self.local_track.depth() > 0 {
            return 0;
        }
        if let Some(prev_token) = self.previous.take() {
            return self.identifier_constant(prev_token);
        }
        unreachable!()
    }

    fn declare_variable(&mut self) {
        if *self.local_track.depth() == 0 {
            return;
        }

        if let Some(prev_token) = self.previous.to_owned() {
            for idx in (0..self.local_track.local_count).rev() {
                if let Some(ref local) = self.local_track.locals[idx as usize] {
                    if local.depth < Some(*self.local_track.depth()) {
                        break;
                    }

                    if prev_token.is_equal(&local.name) {
                        self.error("Already a variable with this name in this scope.");
                    }
                }
            }

            self.add_local(prev_token.to_owned());
        }
    }

    /// outputs the bytecode instruction that defines the new variable and stores its initial value.
    fn define_variable(&mut self, global: u8) {
        if *self.local_track.depth() > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_bytes(OpCode::DefineGlobal.into(), global);
    }

    fn mark_initialized(&mut self) {
        if let Some(local_depth) =
            self.local_track.locals[(self.local_track.local_count - 1) as usize].as_mut()
        {
            local_depth.depth = Some(self.local_track.scope_depth);
        }
    }

    fn parse_number(&mut self, _can_assign: bool) {
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

    fn parse_grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RIGHTPAREN, "Expect ')' after expression.");
    }

    fn parse_unary(&mut self, _can_assign: bool) {
        if let Some(operator_type) = self.previous.take() {
            self.parse_precedence(Precedence::UNARY);
            match operator_type.token_type {
                TokenType::MINUS => self.emit_byte(OpCode::NEGATE as u8),
                TokenType::BANG => self.emit_byte(OpCode::NOT as u8),
                _ => (),
            }
        }
    }

    fn parse_binary(&mut self, _can_assign: bool) {
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

    fn parse_literal(&mut self, _can_assign: bool) {
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

        if let Some(prev_token) = self.previous.as_ref() {
            match Self::get_rule(prev_token.token_type).prefix {
                Some(prefix_rule) => {
                    let can_assign = precedence as u8 <= Precedence::ASSIGNMENT as u8;
                    prefix_rule(self, can_assign);
                    while precedence as u8
                        <= Self::get_rule(self.current.as_ref().unwrap().clone().token_type)
                            .precedence as u8
                    {
                        self.advance();
                        if let Some(infix_rule) =
                            Self::get_rule(self.previous.as_ref().unwrap().clone().token_type).infix
                        {
                            infix_rule(self, can_assign);
                        }
                        if can_assign && self.match_token(TokenType::EQUAL) {
                            self.error("Invalid assignment target.");
                        }
                    }
                }
                None => {
                    self.error("Expect expression");
                }
            }
        }
    }

    fn string(&mut self, _can_assign: bool) {
        if let Some(prev_token) = self.previous.as_mut() {
            let str_value = std::mem::take(&mut prev_token.lexeme);
            let str_obj = ObjectType::ObjString(str_value);
            self.emit_constant(ValueType::Obj(Object::new(str_obj)));
        }
    }

    fn variable(&mut self, can_assign: bool) {
        if let Some(prev_token) = self.previous.as_ref() {
            self.named_variable(prev_token.to_owned(), can_assign);
        }
    }

    fn named_variable(&mut self, token_name: Token, can_assign: bool) {
        let (arg, get_op, set_op) = match self.resolve_local(&token_name) {
            Some(arg) => (arg, OpCode::GetLocal, OpCode::SetLocal),
            None => {
                let new_arg = self.identifier_constant(token_name);
                (new_arg, OpCode::GetGlobal, OpCode::SetGlobal)
            }
        };

        match can_assign && self.match_token(TokenType::EQUAL) {
            true => {
                self.expression();
                self.emit_bytes(set_op.into(), arg);
            }
            false => self.emit_bytes(get_op.into(), arg),
        }
    }

    fn resolve_local(&mut self, token_name: &Token) -> Option<u8> {
        for idx in (0..self.local_track.local_count).rev() {
            if let Some(local) = self.local_track.locals.get(idx as usize) {
                if let Some(local_val) = local.as_ref() {
                    if local_val.name.is_equal(token_name) {
                        if local_val.depth.is_none() {
                            self.error("Can't read local variable in its own initializer.");
                            return None;
                        }
                        return Some(idx);
                    }
                }
            }
        }
        None
    }

    fn begin_scope(&mut self) {
        self.local_track.begin();
    }

    fn end_scope(&mut self) {
        self.local_track.end();

        while let Some(local) = self
            .local_track
            .locals
            .get((self.local_track.local_count.wrapping_sub(1)) as usize)
        {
            match local.as_ref() {
                Some(l) => {
                    if l.depth <= Some(self.local_track.scope_depth) {
                        break;
                    }
                    self.emit_byte(OpCode::POP as u8);
                    self.local_track.local_count -= 1;
                }
                None => break,
            }
        }
    }

    /// takes the given token and adds its lexeme to the chunk’s constant table as a string object.
    fn identifier_constant(&mut self, mut token: Token) -> u8 {
        let str_value = std::mem::take(&mut token.lexeme);
        let str_obj = ObjectType::ObjString(str_value);
        self.make_constant(ValueType::Obj(Object::new(str_obj)))
    }

    fn add_local(&mut self, name: Token) {
        if self.local_track.local_count == 255 {
            self.error("Too many local variables in function.");
            return;
        }

        let local = Local {
            // depth: *self.local_track.depth(),
            depth: None,
            name,
        };

        self.local_track
            .add_local_at_idx(local, self.local_track.local_count);
        self.local_track.local_count += 1;
    }

    /// Adds the ValueType to the chunk->constants and gets the index
    /// Adds the `OpCode::CONSTANT`(u8) and `index` in the chunk->opcodes
    fn emit_constant(&mut self, value: ValueType) {
        let cons_byte_idx = self.make_constant(value);
        self.emit_bytes(OpCode::CONSTANT as u8, cons_byte_idx);
    }

    /// Add the ValueType to chunk->constants and returns the index
    fn make_constant(&mut self, value: ValueType) -> u8 {
        let constant = self.chunk.add_constant(value);

        if constant > 255 {
            self.error("Too many constants in one chunk.");
            return 0;
        }

        constant as u8
    }

    pub fn emit_jump(&mut self, instruction: u8) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);

        self.chunk.op_codes_len() - 2
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.op_codes_len() - offset - 2;

        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        if let Some(first_jump) = self.chunk.op_codes_at_mut(offset) {
            *first_jump = ((jump >> 8) & 0xff) as u8;
        }

        if let Some(next_jump) = self.chunk.op_codes_at_mut(offset + 1) {
            *next_jump = (jump & 0xff) as u8;
        }
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

    /// Checks if the current token doesn't matches the given then returns false
    /// If it matches then, advances the compiler and return true
    fn match_token(&mut self, expected_token: TokenType) -> bool {
        if !self.check_token(expected_token) {
            return false;
        }
        self.advance();
        true
    }

    /// Check if the current token (token_type) matches the expected token
    /// Returns `true` if matches, otherwise `false`
    fn check_token(&mut self, expected_token: TokenType) -> bool {
        if let Some(curr_token) = self.current.as_ref() {
            return curr_token.token_type == expected_token;
        }
        false
    }

    /// If the given token is same as the current token, move forward
    /// Else Error
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
            TokenType::IDENTIFIER => ParseRule {
                prefix: Some(Self::variable),
                infix: None,
                precedence: Precedence::NONE,
            },
            TokenType::STRING => ParseRule {
                prefix: Some(Self::string),
                infix: None,
                precedence: Precedence::NONE,
            },
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

type ParseFn<'scanner, 'chunk> = fn(&mut Compiler<'scanner, 'chunk>, bool) -> ();

#[derive(Default)]
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

pub struct LocalTracking {
    locals: [Option<Local>; 256],
    local_count: u8,
    scope_depth: u8,
}

impl Default for LocalTracking {
    fn default() -> Self {
        LocalTracking {
            locals: [const { None }; 256],
            local_count: 0,
            scope_depth: 0,
        }
    }
}

impl LocalTracking {
    pub fn add_local_at_idx(&mut self, local: Local, idx: u8) {
        self.locals[idx as usize] = Some(local);
    }

    pub fn begin(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end(&mut self) {
        self.scope_depth -= 1;
    }

    pub fn depth(&self) -> &u8 {
        &self.scope_depth
    }
}

pub struct Local {
    name: Token,
    depth: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(u8)]
pub enum Precedence {
    #[default]
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
