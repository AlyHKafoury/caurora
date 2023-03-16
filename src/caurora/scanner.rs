use std::collections::HashMap;

use super::token::{Token, TokenType};

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<String, TokenType>,
    pub error_msg: String,
}

impl Scanner<'_> {
    pub fn new(src: &'static str) -> Self {
        let mut keywords = HashMap::<String, TokenType>::new();
        keywords.insert("and".to_owned(), TokenType::And);
        keywords.insert("class".to_owned(), TokenType::Class);
        keywords.insert("else".to_owned(), TokenType::Else);
        keywords.insert("false".to_owned(), TokenType::False);
        keywords.insert("for".to_owned(), TokenType::For);
        keywords.insert("function".to_owned(), TokenType::Fun);
        keywords.insert("if".to_owned(), TokenType::If);
        keywords.insert("nil".to_owned(), TokenType::Nil);
        keywords.insert("or".to_owned(), TokenType::Or);
        keywords.insert("print".to_owned(), TokenType::Print);
        keywords.insert("return".to_owned(), TokenType::Return);
        keywords.insert("super".to_owned(), TokenType::Super);
        keywords.insert("this".to_owned(), TokenType::This);
        keywords.insert("true".to_owned(), TokenType::True);
        keywords.insert("var".to_owned(), TokenType::Var);
        keywords.insert("while".to_owned(), TokenType::While);
        Scanner {
            source: src,
            start: 0,
            current: 0,
            line: 0,
            error_msg: "".to_owned(),
            keywords,
        }
    }

    fn make_token(&self, t: TokenType) -> Token {
        Token {
            tokentype: t,
            start: self.start,
            length: self.current - self.start,
            line: self.line,
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        return self.source.chars().nth(self.current - 1).unwrap();
    }

    fn token_match(&mut self, expected: char) -> bool {
        if self.at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        return true;
    }

    fn peek(&self) -> char {
        if self.at_end() {
            return '\0';
        }
        return self.source.chars().nth(self.current).unwrap();
    }

    pub fn scan_token(&mut self) -> Token {
        self.start = self.current;

        if self.at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        match c {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            ';' => self.make_token(TokenType::SemiColon),
            '*' => self.make_token(TokenType::Star),
            '!' => match self.token_match('=') {
                true => self.make_token(TokenType::BangEqual),
                false => self.make_token(TokenType::Bang),
            },
            '=' => match self.token_match('=') {
                true => self.make_token(TokenType::EqualEqual),
                false => self.make_token(TokenType::Equal),
            },
            '<' => match self.token_match('=') {
                true => self.make_token(TokenType::LessEqual),
                false => self.make_token(TokenType::Less),
            },
            '>' => match self.token_match('=') {
                true => self.make_token(TokenType::GreaterEqual),
                false => self.make_token(TokenType::Greater),
            },
            '/' => match self.token_match('/') {
                true => {
                    while self.peek() != '\n' && !self.at_end() {
                        self.advance();
                    }
                    self.make_token(TokenType::WhiteSpace)
                }
                false => self.make_token(TokenType::Slash),
            },
            ' ' | '\r' | '\t' => self.make_token(TokenType::WhiteSpace),
            '\n' => {
                self.line += 1;
                self.make_token(TokenType::NewLine)
            }
            '"' => self.read_string(),
            _ => {
                if self.is_digit(c) {
                    self.number()
                } else if self.is_alpha(c) {
                    self.identifier()
                } else {
                    self.error_msg = format!("line : {} , unexpected character {}", self.line, c);
                    self.make_token(TokenType::Error)
                }
            }
        }
    }

    fn at_end(&self) -> bool {
        self.current >= self.source.chars().count()
    }

    fn is_digit(&self, c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    fn number(&mut self) -> Token {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }
        self.make_token(TokenType::Number)
    }

    fn peek_next(&self) -> char {
        if (self.current + 1) >= self.source.chars().count() {
            return '\0';
        }
        return self.source.chars().nth(self.current + 1).unwrap();
    }

    fn is_alpha(&self, c: char) -> bool {
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c == '_');
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        return self.is_alpha(c) || self.is_digit(c);
    }

    fn identifier(&mut self) -> Token {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }
        let text: String = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect();
        let tokentype = self.keywords.get(&text).unwrap_or(&TokenType::Identifier);
        self.make_token(tokentype.to_owned())
    }

    fn read_string(&mut self) -> Token {
        while self.peek() != '"' && !self.at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.at_end() {
            self.error_msg = format!("Unterminated string at {}", self.line);
            return self.make_token(TokenType::Error);
        }

        self.advance();

        self.make_token(TokenType::String)
    }
}
