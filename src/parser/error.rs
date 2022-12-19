use std::{error::Error, fmt};

use crate::lexer::token::Token;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token),
    UnterminatedList,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            ParseError::UnexpectedToken(token) => format!("unexpected `{token}`"),
            ParseError::UnterminatedList => "unterminated list".to_owned(),
        };
        write!(f, "parse error: {}", err)
    }
}
