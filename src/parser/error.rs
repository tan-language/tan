use std::{error::Error, fmt};

use crate::lexer::token::Token;

// #TODO Consider just having a single `ParseError` or `SyntaxError`?

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token),
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            ParseError::UnexpectedToken(token) => format!("unexpected `{token}`"),
        };
        write!(f, "parse error: {}", err)
    }
}
