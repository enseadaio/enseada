use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter};

#[cfg(feature = "serde")]
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Error;
use crate::lexer::Lexer;
use crate::parser::Item;
use crate::parser::Parser;

#[derive(Debug, Eq)]
pub struct Version {
    value: String,
    items: Item,
}

impl Version {
    pub fn parse<S: AsRef<str>>(value: S) -> Result<Self, Error> {
        let value = value.as_ref().to_lowercase();
        let mut lexer = Lexer::new(&value);
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

    pub fn is_snapshot(&self) -> bool {
        self.items
            .find(&|item| match item {
                Item::String(s) => s == "snapshot",
                _ => false,
            })
            .is_some()
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.value, f)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(D::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.items.cmp(&other.items)
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

    #[rstest(
    first,
    other,
    ord,
    case("1", "1.0", Ordering::Equal),                // 1
    case("1", "1-ga", Ordering::Equal),               // 2
    case("1-alpha", "1-a", Ordering::Equal),          // 3
    case("1-beta", "1-b", Ordering::Equal),           // 4
    case("1-milestone", "1-m", Ordering::Equal),      // 5
    case("1-rc", "1-cr", Ordering::Equal),            // 6
    case("1-ga", "1-final", Ordering::Equal),         // 7
    case("1", "1-final", Ordering::Equal),            // 8
    case("1.0", "2.0", Ordering::Less),               // 9
    case("3.0-alpha", "1.0", Ordering::Greater),      // 10
    case("1-alpha.1", "1-alpha.2", Ordering::Less),   // 11
    case("1-beta.1", "1-alpha.2", Ordering::Greater), // 12
    case("1-snapshot", "1-pippo", Ordering::Less)     // 13
    )]
    fn it_compares_versions(first: &str, other: &str, ord: Ordering) {
        let v1 = Version::parse(first).unwrap();
        let v2 = Version::parse(other).unwrap();
        assert_eq!(ord, v1.cmp(&v2))
    }

    #[rstest(input, is, case("1.0", false), case("1.0-snapshot", true))]
    fn it_checks_for_snapshots(input: &str, is: bool) {
        let v = Version::parse(input).unwrap();

        assert_eq!(is, v.is_snapshot());
    }
}
