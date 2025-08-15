use anyhow::{Result, bail};

use crate::{
    token::{Token, TokenKind},
};

pub fn scan(input: &str) -> Result<Vec<Token>> {
    Scanner::new(input).scan()
}

struct Scanner {
    input: Vec<char>,
    current: usize,
}

impl Scanner {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_string().chars().collect(),
            current: 0,
        }
    }

    fn scan(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let next_token = self.scan_next_token()?;
            let is_end = matches!(next_token.kind, TokenKind::EndOfFile);
            tokens.push(next_token);

            if is_end {
                break;
            }
        }

        Ok(tokens)
    }

    fn scan_next_token(&mut self) -> Result<Token> {
        self.consume_whitespace();

        if self.is_at_end() {
            return self.token(TokenKind::EndOfFile);
        }

        let ch = self.eat_next_token();
        match ch {
            '+' => self.token(TokenKind::Plus),
            ch if ch.is_numeric() => {
                let mut literal = String::from(ch);
                while !self.is_at_end() && self.next().is_numeric() {
                    literal.push(self.next());
                    self.advance()
                }
                let number = literal.parse::<i32>()?;
                self.token(TokenKind::Number(number))
            }
            _ => bail!("scanner: unrecognized token: '{}'", ch),
        }
    }

    fn consume_whitespace(&mut self) {
        while !self.is_at_end() && self.next().is_whitespace() {
            self.advance()
        }
    }

    fn token(&self, kind: TokenKind) -> Result<Token> {
        Ok(Token { kind })
    }

    fn eat_next_token(&mut self) -> char {
        let c = self.next();
        self.advance();
        c
    }

    fn next(&self) -> char {
        self.input[self.current]
    }

    fn advance(&mut self) {
        self.current += 1
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }
}
