use std::{error::Error, fmt};

use crate::lexer::token::Token;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token),
    UnterminatedList,
    MalformedAnnotation(String),
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            ParseError::UnexpectedToken(token) => format!("unexpected `{token}`"),
            ParseError::UnterminatedList => "unterminated list".to_owned(),
            ParseError::MalformedAnnotation(ann) => format!("malformed annotation `{ann}`"),
        };
        write!(f, "parse error: {}", err)
    }
}
