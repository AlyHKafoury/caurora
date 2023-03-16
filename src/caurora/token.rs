use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenType{
  // Single-character tokens.
  LeftParen, RightParen, LeftBrace, RightBrace,
  Comma, Dot, Minus, Plus, SemiColon, Slash, Star,

  // One or two character tokens.
  Bang, BangEqual,
  Equal, EqualEqual,
  Greater, GreaterEqual,
  Less, LessEqual,

  // Literals.
  Identifier, String, Number,

  // Keywords.
  And, Class, Else, False, Fun, For, If, Nil, Or,
  Print, Return, Super, This, True, Var, While,

  Eof, Error, WhiteSpace, NewLine
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }    
}

#[derive(Debug,Clone, PartialEq, PartialOrd, Copy)]
pub struct Token {
    pub tokentype: TokenType,
    pub start: usize,
    pub length: usize,
    pub line: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Token Type: {},  Start: {}, Length: {}, Line: {}", self.tokentype, self.start, self.length, self.line)
    }
}