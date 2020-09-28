use std::fmt::{self, Debug, Display, Formatter};

use crate::error::Error;
use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Item {
    Integer(u64),
    String(String),
    List(Vec<Item>),
    Null,
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut visitor = ItemVisitor::default();
        visitor.add(self);
        std::fmt::Display::fmt(&visitor.render(), f)
    }
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn parse(self) -> Result<Item, Error> {
        if self.tokens.is_empty() {
            return Ok(Item::Null);
        }

        if !self.tokens.first().unwrap().is_int() {
            return Err(Error::Parse(format!(
                "Maven versions must begin with an integer"
            )));
        }

        Ok(Self::parse_list(&mut self.tokens.into_iter()))
    }

    fn parse_list<I: Iterator<Item = Token>>(tokens: &mut I) -> Item {
        let mut root = Vec::new();
        let mut buf = String::new();
        let mut prev = None;
        while let Some(token) = tokens.next() {
            if prev.is_none() {
                prev = Some(token.clone());
            }

            match token {
                Token::Integer(i) => {
                    let i = std::char::from_digit(i as u32, 10).unwrap();
                    if let Some(Token::Integer(_)) | None = prev {
                        buf.push(i);
                    } else {
                        if !buf.is_empty() {
                            root.push(Self::parse_item(&buf));
                        }
                        buf = i.to_string();
                    }
                    prev = Some(token)
                }
                Token::Char(c) => {
                    match prev {
                        Some(Token::Char(_)) | None => {
                            buf.push(c);
                        }
                        Some(Token::Integer(_)) => {
                            if !buf.is_empty() {
                                root.push(Self::parse_item(&buf));
                            }
                            buf = c.to_string();
                        }
                        _ => {}
                    }
                    prev = Some(token)
                }
                Token::Dot => {
                    root.push(Self::parse_item(&buf));
                    buf = String::new();
                }
                Token::Dash => {
                    root.push(Self::parse_item(&buf));
                    buf = String::new();
                    let sub = Self::parse_list(tokens);
                    root.push(sub);
                }
            }
        }
        if !buf.is_empty() {
            root.push(Self::parse_item(&buf));
        }
        Item::List(root)
    }

    fn parse_item(buf: &str) -> Item {
        if let Ok(i) = buf.parse::<u64>() {
            Item::Integer(i)
        } else {
            Item::String(buf.to_string())
        }
    }
}

impl<I: Iterator<Item = Token>> From<I> for Parser {
    fn from(iter: I) -> Self {
        Parser {
            tokens: iter.collect(),
        }
    }
}

#[derive(Default)]
struct ItemVisitor(String);

impl ItemVisitor {
    fn add(&mut self, item: &Item) {
        match item {
            Item::Integer(i) => {
                self.0.push('.');
                self.0.push_str(&i.to_string());
            }
            Item::String(s) => {
                self.0.push('.');
                self.0.push_str(s);
            }
            Item::List(list) => {
                self.0.push('-');
                for item in list {
                    self.add(item);
                }
            }
            Item::Null => {}
        }
    }

    fn render(self) -> String {
        self.0
            .trim_start_matches(&['.', '-'][..])
            .trim_end_matches(&['.', '-'][..])
            .replace("-.", "-")
            .replace("-.", "-")
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::Token;

    use super::*;

    #[test]
    fn it_parses_a_token_list() {
        // 1.0-alpha10.5-c
        let tokens = vec![
            Token::Integer(1),
            Token::Dot,
            Token::Integer(0),
            Token::Dash,
            Token::Char('a'),
            Token::Char('l'),
            Token::Char('p'),
            Token::Char('h'),
            Token::Char('a'),
            Token::Integer(1),
            Token::Integer(0),
            Token::Dot,
            Token::Integer(5),
            Token::Dash,
            Token::Char('c'),
        ];

        let exp_items = Item::List(vec![
            Item::Integer(1),
            Item::Integer(0),
            Item::List(vec![
                Item::String("alpha".to_string()),
                Item::Integer(10),
                Item::Integer(5),
                Item::List(vec![Item::String("c".to_string())]),
            ]),
        ]);

        let parser = Parser::from(tokens.into_iter());
        let items = parser.parse().unwrap();

        assert_eq!(exp_items, items);
    }

    #[test]
    fn it_renders_items_to_string() {
        let items = Item::List(vec![
            Item::Integer(1),
            Item::Integer(0),
            Item::List(vec![
                Item::String("alpha".to_string()),
                Item::Integer(10),
                Item::Integer(5),
                Item::List(vec![Item::String("c".to_string())]),
            ]),
        ]);

        let s = items.to_string();
        assert_eq!("1.0-alpha.10.5-c", s);
    }
}
