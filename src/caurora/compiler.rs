use super::{
    errorlogger::log_error,
    memoryslice::MemorySlice,
    opcodes::OpCode,
    scanner::Scanner,
    token::{Token, TokenType},
    values::{Object, Value},
};

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    pub fn repr(&self) -> u16 {
        // SAFETY: Because `Self` is marked `repr(u16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u16` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u16>() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Local {
    name: Token,
    depth: usize,
}

pub struct Compiler {
    current: Token,
    previous: Token,
    has_error: bool,
    source: &'static str,
    memory: MemorySlice,
    scanner: Scanner<'static>,
    locals: Vec<Local>,
    scope_depth: usize,
}

impl Compiler {
    pub fn new(source: &'static str, memory: MemorySlice, scanner: Scanner<'static>) -> Self {
        Compiler {
            current: Token {
                tokentype: TokenType::Nil,
                start: 0,
                length: 0,
                line: 0,
            },
            previous: Token {
                tokentype: TokenType::Nil,
                start: 0,
                length: 0,
                line: 0,
            },
            has_error: false,
            source,
            memory,
            scanner,
            locals: Vec::<Local>::new(),
            scope_depth: 0,
        }
    }

    pub fn compile(&mut self) -> MemorySlice {
        self.advance();
        while !self.match_token(TokenType::Eof) {
            self.declaration();
        }
        // self.consume(TokenType::Eof, "Expect end of expression.");
        self.memory.push(OpCode::Return);
        self.memory.clone()
    }

    fn number(&mut self, can_assign: bool) {
        let value: String = self
            .source
            .chars()
            .skip(self.previous.start)
            .take(self.previous.length)
            .collect();
        let value = value.parse::<f64>().unwrap();
        self.memory
            .push_constant(OpCode::Constant, Value::Number(value))
    }

    fn get_token_name(&self) -> String {
        self.source
            .chars()
            .skip(self.current.start)
            .take(self.current.length)
            .collect()
    }

    pub fn advance(&mut self) {
        self.previous = self.current;
        loop {
            self.current = self.scanner.scan_token();
            // println!(
            //     "Current Token <{}> {:#?}",
            //     self.get_token_name(),
            //     &self.current
            // );
            match self.current.tokentype {
                TokenType::NewLine => {
                    self.memory.line_end();
                    continue;
                }
                TokenType::WhiteSpace => continue,
                TokenType::Error => {
                    self.has_error = true;
                    log_error(&self.scanner.error_msg)
                }
                _ => break,
            }
        }
    }

    fn grouping(&mut self, can_assign: bool) {
        self.expression();
        self.consume(
            TokenType::RightParen,
            &format!("Expect ')' after expression at {}", self.current),
        );
    }

    fn unary(&mut self, can_assign: bool) {
        let operator = self.previous.tokentype;

        self.parse_precedence(Precedence::Unary.repr());

        match operator {
            TokenType::Minus => self.memory.push(OpCode::Negate),
            TokenType::Bang => self.memory.push(OpCode::Not),
            _ => log_error(&format!(
                "invalid unary operator exptected - or ! at {}",
                self.current
            )),
        }
    }

    fn consume(&mut self, tokentype: TokenType, message: &str) {
        if self.current.tokentype == tokentype {
            self.advance();
        } else {
            panic!(
                "Faild to Consume Correct token type {}, {}, current: {} , prev: {}",
                tokentype, message, self.current, self.previous
            );
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment.repr());
    }

    fn and_op(&mut self) {
        self.logical_op(Precedence::And, OpCode::JmpFalse)
    }

    fn or_op(&mut self) {
        self.logical_op(Precedence::Or, OpCode::JmpTrue)
    }

    fn logical_op(&mut self, prec: Precedence, jmp: OpCode) {
        let end_jmp = self.push_jmp(jmp);

        self.memory.push(OpCode::Pop);
        self.parse_precedence(prec.repr());

        self.patch_address(end_jmp);
    }

    fn infix(&mut self, can_assign: bool) -> Option<()> {
        match self.previous.tokentype {
            TokenType::Minus => self.binary(can_assign),
            TokenType::Plus => self.binary(can_assign),
            TokenType::Slash => self.binary(can_assign),
            TokenType::Star => self.binary(can_assign),
            TokenType::BangEqual => self.binary(can_assign),
            TokenType::EqualEqual => self.binary(can_assign),
            TokenType::Greater => self.binary(can_assign),
            TokenType::GreaterEqual => self.binary(can_assign),
            TokenType::Less => self.binary(can_assign),
            TokenType::LessEqual => self.binary(can_assign),
            TokenType::And => self.and_op(),
            TokenType::Or => self.or_op(),
            _ => {
                return None;
            }
        }
        Some(())
    }

    fn prefix(&mut self, can_assign: bool) -> Option<()> {
        match self.previous.tokentype {
            TokenType::LeftParen => self.grouping(can_assign),
            TokenType::Minus => self.unary(can_assign),
            TokenType::Number => self.number(can_assign),
            TokenType::Nil => self.literal(can_assign),
            TokenType::True => self.literal(can_assign),
            TokenType::False => self.literal(can_assign),
            TokenType::Bang => self.unary(can_assign),
            TokenType::String => self.string(can_assign),
            TokenType::Identifier => self.identifier(can_assign),
            _ => {
                return None;
            }
        }
        Some(())
    }

    fn identifier(&mut self, can_assign: bool) {
        let local_var = self.find_local_var();
        let global_var = self.parse_identifier(self.previous);
        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            if local_var < 0 {
                self.memory.push_constant(
                    OpCode::SetGlobalVar,
                    Value::Object(Object::String(global_var)),
                )
            } else {
                self.memory
                    .push_constant(OpCode::SetLocalVar, Value::Number(local_var as f64))
            }
        } else {
            if local_var < 0 {
                self.memory.push_constant(
                    OpCode::GetGlobalVar,
                    Value::Object(Object::String(global_var)),
                )
            } else {
                self.memory
                    .push_constant(OpCode::GetLocalVar, Value::Number(local_var as f64))
            }
        }
    }

    fn find_local_var(&mut self) -> isize {
        if self.locals.len() == 0 {
            return -1;
        }
        for i in (0..self.locals.len()).rev() {
            if self.parse_identifier(self.locals[i].name) == self.parse_identifier(self.previous) {
                return i.try_into().unwrap();
            }
        }
        -1
    }

    fn parse_precedence(&mut self, precedence: u16) {
        // print!(
        //     "Starting preced at <{}> {:#?}",
        //     self.get_token_name(),
        //     self.current
        // );
        self.advance();
        let can_assign = precedence <= Precedence::Assignment.repr();
        match self.prefix(can_assign) {
            Some(_) => (),
            None => log_error(&format!(
                "Error at token {} not usable as prefix",
                self.previous
            )),
        }
        while precedence <= self.get_rule(self.current.tokentype).repr() {
            self.advance();
            self.infix(can_assign);
        }
        if can_assign && self.match_token(TokenType::Equal) {
            panic!(
                "Invalid assignment at current: {:#?}, prev: {:#?}",
                self.current, self.previous
            )
        }
        // print!(
        //     "Ending preced at <{}> {:#?}",
        //     self.get_token_name(),
        //     self.current
        // );
    }

    fn get_rule(&self, op: TokenType) -> Precedence {
        match op {
            TokenType::Minus => Precedence::Term,
            TokenType::Plus => Precedence::Term,
            TokenType::Slash => Precedence::Factor,
            TokenType::Star => Precedence::Factor,
            TokenType::BangEqual => Precedence::Equality,
            TokenType::EqualEqual => Precedence::Equality,
            TokenType::Greater => Precedence::Comparison,
            TokenType::GreaterEqual => Precedence::Comparison,
            TokenType::Less => Precedence::Comparison,
            TokenType::LessEqual => Precedence::Comparison,
            TokenType::And => Precedence::And,
            TokenType::Or => Precedence::Or,
            _ => Precedence::None,
        }
    }

    fn binary(&mut self, can_assign: bool) {
        let operator = self.previous.tokentype;
        let precendence = self.get_rule(operator);
        self.parse_precedence(precendence.repr() + 1);

        match operator {
            TokenType::Plus => self.memory.push(OpCode::Add),
            TokenType::Minus => self.memory.push(OpCode::Subtract),
            TokenType::Star => self.memory.push(OpCode::Multiply),
            TokenType::Slash => self.memory.push(OpCode::Divide),
            TokenType::BangEqual => {
                self.memory.push(OpCode::Equal);
                self.memory.push(OpCode::Not)
            }
            TokenType::EqualEqual => self.memory.push(OpCode::Equal),
            TokenType::Greater => self.memory.push(OpCode::Greater),
            TokenType::GreaterEqual => {
                self.memory.push(OpCode::Less);
                self.memory.push(OpCode::Not)
            }
            TokenType::Less => self.memory.push(OpCode::Less),
            TokenType::LessEqual => {
                self.memory.push(OpCode::Greater);
                self.memory.push(OpCode::Not)
            }
            _ => log_error(&format!("invalid binary operator at {}", self.current)),
        }
    }

    fn literal(&mut self, can_assign: bool) {
        match self.previous.tokentype {
            TokenType::Nil => self.memory.push(OpCode::Nil),
            TokenType::True => self.memory.push(OpCode::True),
            TokenType::False => self.memory.push(OpCode::False),
            _ => panic!("Invalid literal type {:#?}", self.previous),
        }
    }

    fn string(&mut self, can_assign: bool) {
        let current_string: String = self
            .source
            .chars()
            .skip(self.previous.start + 1)
            .take(self.previous.length - 2)
            .collect();
        self.memory.push_constant(
            OpCode::Constant,
            Value::Object(Object::String(current_string)),
        );
    }

    fn check(&mut self, tokentype: TokenType) -> bool {
        self.current.tokentype == tokentype
    }

    fn match_token(&mut self, tokentype: TokenType) -> bool {
        if !self.check(tokentype) {
            return false;
        }
        self.advance();
        true
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) {
        self.consume(TokenType::Identifier, "expect identifier after var.");
        let local_var = self.previous;
        let global_var = self.parse_identifier(self.previous);

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            self.memory.push(OpCode::Nil);
        }

        self.consume(TokenType::SemiColon, "expect ';' after value.");

        if self.scope_depth > 0 {
            self.local_var(local_var);
            return;
        }

        self.memory.push_constant(
            OpCode::DefineGlobalVar,
            Value::Object(Object::String(global_var)),
        )
    }

    fn local_var(&mut self, name: Token) {
        if self.scope_depth == 0 {
            return;
        }

        self.locals.push(Local {
            name: name,
            depth: self.scope_depth,
        });
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        } else if self.match_token(TokenType::If) {
            self.if_statement();
        } else if self.match_token(TokenType::While) {
            self.while_statement();
        } else if self.match_token(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "expect ')' after condition.");
        let thn_address = self.push_jmp(OpCode::JmpFalse);
        self.statement();
        let else_address = self.push_jmp(OpCode::Jmp);
        self.patch_address(thn_address);

        if self.match_token(TokenType::Else) {
            self.statement();
        }
        self.patch_address(else_address);
        self.memory.push(OpCode::Pop);
    }

    fn while_statement(&mut self) {
        let loop_start = self.memory.get_memory_size();
        println!("LOOOP START {}", loop_start);
        self.consume(TokenType::LeftParen, "expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "expect ')' after condition.");
        
        let end_address = self.push_jmp(OpCode::JmpFalse);
        self.memory.push(OpCode::Pop);
        self.statement();
        self.push_loop(loop_start);

        self.patch_address(end_address);
        self.memory.push(OpCode::Pop);
    }

    fn push_loop(&mut self, loop_start: usize) {
        self.memory.push(OpCode::Loop);
        let steps = self.memory.get_memory_size() - loop_start + 1;
        println!("==============jmping to {} {}", steps, self.memory.get_memory_size());
        self.memory.push_raw(steps as u16);
    }

    fn patch_address(&mut self, jmp_address: usize) {
        let steps = self.memory.get_memory_size() - jmp_address - 1;
        self.memory.replace_at_location(jmp_address, steps as u16)
    }

    fn push_jmp(&mut self, op: OpCode) -> usize {
        self.memory.push(op);
        self.memory.push(OpCode::Panic);
        self.memory.get_memory_size() - 1
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "expect '}' after block.")
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while self.locals.len() > 0 && self.locals.last().unwrap().depth > self.scope_depth {
            self.memory.push(OpCode::Pop);
            self.locals.pop();
        }
    }

    fn parse_identifier(&mut self, token: Token) -> String {
        let var_name: String = self
            .source
            .chars()
            .skip(token.start)
            .take(token.length)
            .collect();
        var_name
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "expect ';' after value.");
        self.memory.push(OpCode::Print)
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "expect ';' after expression.");
        self.memory.push(OpCode::Pop)
    }
}
