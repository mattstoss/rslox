#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Comma,
    Dot,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Minus,
    Plus,
    Semicolon,
    Number(i32),
    EndOfFile,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
}
