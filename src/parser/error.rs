use std::{error::Error, fmt};

use crate::lexer::token::Token;

#[derive(Debug)]
pub enum ParseError {
    InvalidQuote,
    UnexpectedToken(Token),
    UnterminatedList,
    MalformedAnnotation(String),
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            ParseError::InvalidQuote => "invalid quote".to_string(),
            ParseError::UnexpectedToken(token) => format!("unexpected `{token}`"),
            ParseError::UnterminatedList => "unterminated list".to_string(),
            ParseError::MalformedAnnotation(ann) => format!("malformed annotation `{ann}`"),
        };
        write!(f, "parse error: {}", err)
    }
}
