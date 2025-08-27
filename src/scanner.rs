use std::mem;

use anyhow::{Result, bail};

use crate::token::{Token, TokenKind};

pub fn scan(input: &str) -> Result<Vec<Token>> {
    Scanner::new(input).scan()
}

struct Scanner {
    input: Vec<char>,
    current: usize,
    tokens: Vec<Token>,
}

fn equals(ch: char) -> impl Fn(char) -> bool {
    let predicate = move |c| c == ch;
    predicate
}

fn is_numeric() -> impl Fn(char) -> bool {
    let predicate = move |c: char| c.is_numeric();
    predicate
}

fn is_alphabetic() -> impl Fn(char) -> bool {
    let predicate = move |c: char| c.is_alphabetic();
    predicate
}

fn is_whitespace() -> impl Fn(char) -> bool {
    let predicate = move |c: char| c.is_whitespace();
    predicate
}

fn is_not_newline() -> impl Fn(char) -> bool {
    let predicate = move |c: char| c != '\n';
    predicate
}

fn is_not_double_quote() -> impl Fn(char) -> bool {
    let predicate = move |c: char| c != '"';
    predicate
}

impl Scanner {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_string().chars().collect(),
            current: 0,
            tokens: Vec::new(),
        }
    }

    fn scan(&mut self) -> Result<Vec<Token>> {
        loop {
            self.scan_next_token()?;
            if self.is_at_end() {
                self.add_token(TokenKind::EndOfFile);
                return Ok(mem::take(&mut self.tokens));
            }
        }
    }

    fn scan_next_token(&mut self) -> Result<()> {
        self.consume_whitespace();
        if self.is_at_end() {
            return Ok(());
        }

        let ch = self.eat_next();
        match ch {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            ';' => self.add_token(TokenKind::Semicolon),
            '-' => self.add_token(TokenKind::Minus),
            '+' => self.add_token(TokenKind::Plus),
            '*' => self.add_token(TokenKind::Star),
            '/' => match self.try_eat_next(equals('/')) {
                Some(_) => self.consume_single_line_comment(),
                None => self.add_token(TokenKind::Slash),
            },
            '!' => match self.try_eat_next(equals('=')) {
                Some(_) => self.add_token(TokenKind::BangEqual),
                None => self.add_token(TokenKind::Bang),
            },
            '=' => match self.try_eat_next(equals('=')) {
                Some(_) => self.add_token(TokenKind::EqualEqual),
                None => self.add_token(TokenKind::Equal),
            },
            '<' => match self.try_eat_next(equals('=')) {
                Some(_) => self.add_token(TokenKind::LessEqual),
                None => self.add_token(TokenKind::Less),
            },
            '>' => match self.try_eat_next(equals('=')) {
                Some(_) => self.add_token(TokenKind::GreaterEqual),
                None => self.add_token(TokenKind::Greater),
            },
            '"' => {
                let start = self.current;

                self.consume_while(is_not_double_quote());
                if self.is_at_end() {
                    // FIXME: better error message for unterminated strings
                    bail!("unterminated string")
                }
                self.advance();

                let end = self.current - 1;

                let string = self.input[start..end].iter().collect();
                self.add_token(TokenKind::String(string));
            }
            ch if ch.is_numeric() => {
                let mut literal = String::from(ch);
                while let Some(ch) = self.try_eat_next(is_numeric()) {
                    literal.push(ch);
                }
                let number = literal.parse::<i32>()?;
                self.add_token(TokenKind::Number(number))
            }
            ch if ch.is_alphabetic() => {
                let mut word = String::from(ch);
                while let Some(ch) = self.try_eat_next(is_alphabetic()) {
                    word.push(ch);
                }

                match word.as_str() {
                    "and" => self.add_token(TokenKind::And),
                    "class" => self.add_token(TokenKind::Class),
                    "else" => self.add_token(TokenKind::Else),
                    "false" => self.add_token(TokenKind::False),
                    "for" => self.add_token(TokenKind::For),
                    "fun" => self.add_token(TokenKind::Fun),
                    "if" => self.add_token(TokenKind::If),
                    "nil" => self.add_token(TokenKind::Nil),
                    "or" => self.add_token(TokenKind::Or),
                    "print" => self.add_token(TokenKind::Print),
                    "return" => self.add_token(TokenKind::Return),
                    "super" => self.add_token(TokenKind::Super),
                    "this" => self.add_token(TokenKind::This),
                    "true" => self.add_token(TokenKind::True),
                    "var" => self.add_token(TokenKind::Var),
                    "while" => self.add_token(TokenKind::While),
                    _ => bail!("unrecognized keyword: {}", word),
                }
            }
            _ => bail!("scanner: unrecognized token: '{}'", ch),
        }

        Ok(())
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(is_whitespace())
    }

    fn consume_single_line_comment(&mut self) {
        self.consume_while(is_not_newline())
    }

    fn consume_while(&mut self, predicate: impl Fn(char) -> bool) {
        while self.try_eat_next(&predicate).is_some() {}
    }

    fn add_token(&mut self, kind: TokenKind) {
        let new_token = Token { kind };
        self.tokens.push(new_token)
    }

    fn eat_next(&mut self) -> char {
        let c = self.next();
        self.advance();
        c
    }

    fn try_eat_next(&mut self, predicate: impl Fn(char) -> bool) -> Option<char> {
        if self.is_at_end() {
            return None;
        }

        let ch = self.next();
        if predicate(ch) {
            self.advance();
            return Some(ch);
        }

        None
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
        expected: std::string::String,
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
                assertion: error_msg_eq!("unrecognized keyword: d"),
            },
        )
    }

    #[test]
    fn string_literal() {
        run_tests!(
            TestCase {
                name: "success - single char string",
                input: r#"
                    "a"
                "#,
                assertion: token_kinds_eq!(String("a".to_string()), EndOfFile),
            },
            TestCase {
                name: "success - multi char string",
                input: r#"
                    "test string"
                "#,
                assertion: token_kinds_eq!(String("test string".to_string()), EndOfFile),
            },
            TestCase {
                name: "success - string with numbers and symbols",
                input: r#"
                    "abc !@ 会意 3ab.d"
                "#,
                assertion: token_kinds_eq!(String("abc !@ 会意 3ab.d".to_string()), EndOfFile),
            },
            TestCase {
                name: "success - string concat",
                input: r#"
                    "a" + "bbb"
                "#,
                assertion: token_kinds_eq!(
                    String("a".to_string()),
                    Plus,
                    String("bbb".to_string()),
                    EndOfFile
                ),
            },
            TestCase {
                name: "failure - unterminated string literal",
                input: r#"
                    "a + 3
                "#,
                assertion: error_msg_eq!("unterminated string"),
            },
        )
    }

    #[test]
    fn addition_stress_test() {
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
    fn single_character() {
        run_tests!(
            TestCase {
                name: "success - parenthesis",
                input: "()",
                assertion: token_kinds_eq!(LeftParen, RightParen, EndOfFile),
            },
            TestCase {
                name: "success - brace",
                input: "{}",
                assertion: token_kinds_eq!(LeftBrace, RightBrace, EndOfFile),
            },
            TestCase {
                name: "success - punctuation",
                input: ",.;",
                assertion: token_kinds_eq!(Comma, Dot, Semicolon, EndOfFile),
            },
            TestCase {
                name: "success - operators",
                input: "-+/*",
                assertion: token_kinds_eq!(Minus, Plus, Slash, Star, EndOfFile),
            },
        )
    }

    #[test]
    fn multi_character() {
        run_tests!(
            TestCase {
                name: "success - bang equal",
                input: "!=!",
                assertion: token_kinds_eq!(BangEqual, Bang, EndOfFile),
            },
            TestCase {
                name: "success - equal equal",
                input: "===",
                assertion: token_kinds_eq!(EqualEqual, Equal, EndOfFile),
            },
            TestCase {
                name: "success - less equal",
                input: "<<==",
                assertion: token_kinds_eq!(Less, LessEqual, Equal, EndOfFile),
            },
            TestCase {
                name: "success - greater equal",
                input: ">>==",
                assertion: token_kinds_eq!(Greater, GreaterEqual, Equal, EndOfFile),
            },
        )
    }

    #[test]
    fn code_comments() {
        run_tests!(
            TestCase {
                name: "success - division",
                input: "/",
                assertion: token_kinds_eq!(Slash, EndOfFile)
            },
            TestCase {
                name: "success - single line comment",
                input: "// single line comment",
                assertion: token_kinds_eq!(EndOfFile)
            },
            TestCase {
                name: "success - single line comment with slash",
                input: r#"
                    /
                    // comment
                    /
                "#,
                assertion: token_kinds_eq!(Slash, Slash, EndOfFile)
            },
        )
    }

    #[test]
    fn keyword() {
        run_tests!(
            TestCase {
                name: "success - all keywords",
                input: r#"
                    and
                    class
                    else
                    false
                    for
                    fun
                    if
                    nil
                    or
                    print
                    return
                    super
                    this
                    true
                    var
                    while
                "#,
                assertion: token_kinds_eq!(
                    And, Class, Else, False, For, Fun, If, Nil, Or, Print, Return, Super, This,
                    True, Var, While, EndOfFile
                )
            },
            TestCase {
                name: "success - keyword expression",
                input: "3 and true",
                assertion: token_kinds_eq!(Number(3), And, True, EndOfFile)
            },
            TestCase {
                name: "failure - invalid keyword",
                input: "d",
                assertion: error_msg_eq!("unrecognized keyword: d"),
            },
        )
    }

    #[test]
    fn fails_on_unrecognized_input() {
        run_tests!(TestCase {
            name: "failures - unrecognized single character",
            input: "?",
            assertion: error_msg_eq!("unrecognized token: '?'"),
        });
    }
}
