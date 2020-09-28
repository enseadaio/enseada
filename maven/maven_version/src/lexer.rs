use std::fmt::{self, Display, Formatter};
use std::str::Chars;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Integer(u64),
    Char(char),
    Dot,
    Dash,
}

impl Token {
    pub fn is_int(&self) -> bool {
        if let Token::Integer(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct UnexpectedChar(pub char);

impl Display for UnexpectedChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        format!("unsupported character '{}'", self.0).fmt(f)
    }
}

impl std::error::Error for UnexpectedChar {}

#[derive(Debug)]
pub struct Lexer<'i> {
    input: &'i str,
    chars: Chars<'i>,
    current: Option<char>,
}

impl<'i> Lexer<'i> {
    pub fn new(input: &'i str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        Lexer {
            input,
            chars,
            current,
        }
    }

    pub fn rewind(&mut self) {
        let mut chars = self.input.chars();
        let current = chars.next();
        self.chars = chars;
        self.current = current;
    }

    fn shift(&mut self) {
        self.current = self.chars.next();
    }
}

impl<'i> Iterator for Lexer<'i> {
    type Item = Result<Token, UnexpectedChar>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.current {
            let t = match c {
                '.' => Token::Dot,
                '-' => Token::Dash,
                '0'..='9' => Token::Integer(c.to_digit(10).unwrap() as u64),
                'a'..='z' | 'A'..='Z' => Token::Char(c),
                c => return Some(Err(UnexpectedChar(c))),
            };
            self.shift();
            Some(Ok(t))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_a_version() {
        let version = "1.0-alpha.1";
        let exp_tokens = vec![
            Token::Integer(1),
            Token::Dot,
            Token::Integer(0),
            Token::Dash,
            Token::Char('a'),
            Token::Char('l'),
            Token::Char('p'),
            Token::Char('h'),
            Token::Char('a'),
            Token::Dot,
            Token::Integer(1),
        ];

        let lexer = Lexer::new(version);
        let tokens: Vec<Token> = lexer.into_iter().map(Result::unwrap).collect();
        assert_eq!(exp_tokens, tokens);
    }
}
