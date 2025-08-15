#[derive(Debug)]
pub enum TokenKind {
    Plus,
    Number(i32),
    EndOfFile,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
}
