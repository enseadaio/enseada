use crate::error::Error;
use crate::lexer::{Lexer, Token, UnexpectedChar};
use crate::parser::Item;
use crate::parser::Parser;

mod error;
mod lexer;
mod parser;

pub struct Version {
    value: String,
    items: Item,
}

impl Version {
    pub fn parse<S: AsRef<str>>(value: S) -> Result<Self, Error> {
        let value = value.as_ref();
        let mut lexer = Lexer::new(value);
        if lexer.any(|res| res.is_err()) {
            return Err(Error::Parse(format!("invalid Maven version: '{}'", value)));
        }
        lexer.rewind();
        let tokens = lexer.filter_map(|res| match res {
            Ok(token) => Some(token),
            Err(_) => None,
        });

        let parser = Parser::from(tokens);
        Ok(Version {
            value: value.to_string(),
            items: parser
                .parse()
                .map_err(|err| Error::Parse(format!("{}, was '{}'", err, value)))?,
        })
    }
}

#[cfg(test)]
mod test {
    use rstest::*;

    use super::*;

    #[rstest(
        input,
        case("1.0"),
        case("3.0-alpha"),
        case("1.0-beta.1-c"),
        case("1-alpha.15-12.t")
    )]
    fn it_parses_a_valid_version_string(input: &str) {
        let v = Version::parse(input).unwrap();

        assert_eq!(input, v.value);
        assert_eq!(input, v.items.to_string())
    }

    #[rstest(input, case("alpha-1.0"), case("-3.0-alpha"), case("?_+="))]
    fn it_errors_with_an_invalid_version_string(input: &str) {
        let res = Version::parse(input);

        assert!(res.is_err());
    }
}
