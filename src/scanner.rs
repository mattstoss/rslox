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

    struct TestCase {
        name: &'static str,
        input: &'static str,
        assertion: Box<dyn Matcher>,
    }

    trait Matcher {
        fn check(&self, result: &Result<Vec<Token>>) -> Result<()>;
    }

    struct TokenKindMatcher {
        expected: Vec<TokenKind>,
    }

    impl Matcher for TokenKindMatcher {
        fn check(&self, result: &Result<Vec<Token>>) -> Result<()> {
            match result {
                Ok(tokens) => {
                    let actual: Vec<TokenKind> =
                        tokens.into_iter().map(|t| t.kind.clone()).collect();

                    if actual == self.expected {
                        Ok(())
                    } else {
                        bail!(
                            "Token kinds did not match.\nExpected: {:?}\n  Actual: {:?}",
                            self.expected,
                            actual
                        );
                    }
                }
                Err(e) => {
                    bail!("Expected success, but the scan failed with: {}", e);
                }
            }
        }
    }

    struct ErrorMsgMatcher {
        expected: String,
    }

    impl Matcher for ErrorMsgMatcher {
        fn check(&self, result: &Result<Vec<Token>>) -> Result<()> {
            match result {
                Ok(_) => {
                    bail!("Expected a scan error, but the operation succeeded.");
                }
                Err(e) => {
                    let actual_msg = e.to_string();
                    if actual_msg.contains(&self.expected) {
                        Ok(())
                    } else {
                        bail!(
                            "Error message did not match.\nExpected to contain: \"{}\"\n           Actual: \"{}\"",
                            self.expected,
                            actual_msg
                        );
                    }
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

    fn run_test_internal(test_cases: &[TestCase]) {
        for tc in test_cases {
            let scan_result = scan(tc.input); // The input is used here...

            let check_result = tc.assertion.check(&scan_result);

            if let Err(error_message) = check_result {
                panic!(
                    "\n\n- Test Case Failed: '{}'\n- Input: '{}'\n- Reason: {}\n\n",
                    tc.name, tc.input, error_message
                );
            }
        }
    }

    macro_rules! run_tests {
        ($($test_case:expr),* $(,)?) => {
            run_test_internal(&[$($test_case),*])
        };
    }

    #[test]
    fn ignores_whitespace() {
        run_tests!(
            TestCase {
                name: "success - only spaces",
                input: "  ",
                assertion: token_kinds_eq!(EndOfFile),
            },
            TestCase {
                name: "success - only tabs",
                input: "\t\t",
                assertion: token_kinds_eq!(EndOfFile),
            },
            TestCase {
                name: "success - only newlines",
                input: "\n\n",
                assertion: token_kinds_eq!(EndOfFile),
            },
            TestCase {
                name: "success - only carriage returns",
                input: "\r\r",
                assertion: token_kinds_eq!(EndOfFile),
            },
            TestCase {
                name: "success - mixed whitespace",
                input: " \t \n \r ",
                assertion: token_kinds_eq!(EndOfFile),
            },
            TestCase {
                name: "success - leading whitespace",
                input: "  + ",
                assertion: token_kinds_eq!(Plus, EndOfFile),
            },
            TestCase {
                name: "success - trailing whitespace",
                input: "+  ",
                assertion: token_kinds_eq!(Plus, EndOfFile),
            },
            TestCase {
                name: "success - surrounding whitespace",
                input: " +  ",
                assertion: token_kinds_eq!(Plus, EndOfFile),
            },
            TestCase {
                name: "success - whitespace between numbers",
                input: "10 + 20",
                assertion: token_kinds_eq!(Number(10), Plus, Number(20), EndOfFile),
            },
            TestCase {
                name: "success - mixed whitespace between numbers",
                input: " 10\t+\n20\r ",
                assertion: token_kinds_eq!(Number(10), Plus, Number(20), EndOfFile),
            },
            TestCase {
                name: "success - empty input",
                input: "",
                assertion: token_kinds_eq!(EndOfFile),
            },
        )
    }

    #[test]
    fn number_literal() {
        run_tests!(
            TestCase {
                name: "success - single digit",
                input: "4",
                assertion: token_kinds_eq!(Number(4), EndOfFile),
            },
            TestCase {
                name: "success - multiple digits",
                input: "44",
                assertion: token_kinds_eq!(Number(44), EndOfFile),
            },
            TestCase {
                name: "success - zero",
                input: "0",
                assertion: token_kinds_eq!(Number(0), EndOfFile),
            },
            TestCase {
                name: "success - max i32 value",
                input: "2147483647",
                assertion: token_kinds_eq!(Number(i32::MAX), EndOfFile),
            },
            TestCase {
                name: "success - numbers in an expression",
                input: "12 + 345",
                assertion: token_kinds_eq!(Number(12), Plus, Number(345), EndOfFile),
            },
            TestCase {
                name: "failure - invalid character after a number",
                input: "0d",
                assertion: error_msg_eq!("unrecognized token: 'd'"),
            },
        )
    }

    #[test]
    fn addition() {
        run_tests!(
            TestCase {
                name: "success - simple addition",
                input: "8 + 2",
                assertion: token_kinds_eq!(Number(8), Plus, Number(2), EndOfFile),
            },
            TestCase {
                name: "success - chained addition",
                input: "8 + 2 + 1",
                assertion: token_kinds_eq!(Number(8), Plus, Number(2), Plus, Number(1), EndOfFile),
            },
            TestCase {
                name: "success - multi-digit addition",
                input: "882 + 2",
                assertion: token_kinds_eq!(Number(882), Plus, Number(2), EndOfFile),
            },
        )
    }

    #[test]
    fn errors() {
        run_tests!(TestCase {
            name: "failures - unrecognized single character",
            input: "?",
            assertion: error_msg_eq!("unrecognized token: '?'"),
        });
    }
}
