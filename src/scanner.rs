use anyhow::{Result, bail};

use crate::token::{Token, TokenKind};

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

#[cfg(test)]
mod tests {
    use super::*;
    use TokenKind::*;

    trait Matcher {
        fn check(&self, input: &str, result: &Result<Vec<Token>>);
    }

    struct TokenKindMatcher {
        expected: Vec<TokenKind>,
    }

    impl Matcher for TokenKindMatcher {
        fn check(&self, input: &str, result: &Result<Vec<Token>>) {
            match result {
                Ok(tokens) => {
                    let actual: Vec<TokenKind> =
                        tokens.into_iter().map(|t| t.kind.clone()).collect();
                    assert_eq!(actual, self.expected, "Failed on input: \"{}\"", input);
                }
                Err(e) => panic!(
                    "Failed on input: \"{}\". Expected success but got error: {}",
                    input, e
                ),
            }
        }
    }

    struct ErrorMsgMatcher {
        expected: String,
    }

    impl Matcher for ErrorMsgMatcher {
        fn check(&self, input: &str, result: &Result<Vec<Token>>) {
            match result {
                Ok(_) => panic!(
                    "Failed on input: \"{}\". Expected error but got success.",
                    input
                ),
                Err(e) => {
                    let actual_msg = e.to_string();
                    assert!(
                        actual_msg.contains(&self.expected),
                        "Failed on input: \"{}\". Expected error '{}', but got '{}'",
                        input,
                        self.expected,
                        actual_msg
                    );
                }
            }
        }
    }

    fn create_kinds_matcher(expected: &[TokenKind]) -> Box<dyn Matcher> {
        Box::new(TokenKindMatcher {
            expected: expected.to_vec(),
        })
    }

    fn create_error_matcher(expected: &str) -> Box<dyn Matcher> {
        Box::new(ErrorMsgMatcher {
            expected: expected.to_string(),
        })
    }

    macro_rules! token_kinds_eq {
        ($($kind:expr),*) => {
            create_kinds_matcher(&[$($kind),*])
        };
    }

    macro_rules! error_msg_eq {
        ($msg:expr) => {
            create_error_matcher($msg)
        };
    }

    fn run_test(test_cases: &[(&str, Box<dyn Matcher>)]) {
        for (input, matcher) in test_cases {
            let result = scan(input);
            matcher.check(input, &result);
        }
    }

    #[test]
    fn ignores_whitespace() {
        run_test(&[
            // === Different Whitespace Characters ===
            ("    ", token_kinds_eq!(EndOfFile)),
            ("\t\t", token_kinds_eq!(EndOfFile)),
            ("\n\n", token_kinds_eq!(EndOfFile)),
            ("\r\r", token_kinds_eq!(EndOfFile)),
            (" \t \n \r ", token_kinds_eq!(EndOfFile)),
            // === Whitespace Around Tokens ===
            ("   + ", token_kinds_eq!(Plus, EndOfFile)),
            ("+   ", token_kinds_eq!(Plus, EndOfFile)),
            ("  +  ", token_kinds_eq!(Plus, EndOfFile)),
            (
                "10 + 20",
                token_kinds_eq!(Number(10), Plus, Number(20), EndOfFile),
            ),
            (
                " 10\t+\n20\r ",
                token_kinds_eq!(Number(10), Plus, Number(20), EndOfFile),
            ),
            // === Edge Cases ===
            ("", token_kinds_eq!(EndOfFile)),
        ])
    }

    #[test]
    fn number_literal() {
        run_test(&[
            // === Basic Cases ===
            ("4", token_kinds_eq!(Number(4), EndOfFile)),
            ("44", token_kinds_eq!(Number(44), EndOfFile)),
            // === Edge Cases ===
            ("0", token_kinds_eq!(Number(0), EndOfFile)),
            ("2147483647", token_kinds_eq!(Number(i32::MAX), EndOfFile)),
            // === Contextual Cases ===
            (
                "12 + 345",
                token_kinds_eq!(Number(12), Plus, Number(345), EndOfFile),
            ),
            // === Error cases ===
            ("0ddd", error_msg_eq!("unrecognized token: 'd'")),
        ])
    }

    #[test]
    fn addition() {
        run_test(&[
            // Single add
            (
                "8 + 2",
                token_kinds_eq!(Number(8), Plus, Number(2), EndOfFile),
            ),
            // Multi add
            (
                "8 + 2 + 1",
                token_kinds_eq!(Number(8), Plus, Number(2), Plus, Number(1), EndOfFile),
            ),
            // Multidigit add
            (
                "882 + 2",
                token_kinds_eq!(Number(882), Plus, Number(2), EndOfFile),
            ),
        ])
    }

    #[test]
    fn errors() {
        run_test(&[("?", error_msg_eq!("unrecognized token: '?'"))]);
    }
}
