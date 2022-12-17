use std::{error::Error, fmt};

// #TODO Consider just having a single `ParseError` or `SyntaxError`?

#[derive(Debug)]
pub enum ParseError {
    NoToken,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            ParseError::NoToken => "no token".to_owned(),
        };
        write!(f, "parse error: {}", err)
    }
}
