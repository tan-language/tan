use std::fmt;
use std::num::{ParseFloatError, ParseIntError};

// #TODO consider renaming to `LexError`.

#[derive(Debug)]
pub enum LexicalError {
    UnexpectedEol,
    MalformedInt(ParseIntError),
    MalformedFloat(ParseFloatError),
    UnterminatedString,
    UnterminatedAnnotation,
}

impl std::error::Error for LexicalError {}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexicalError::UnexpectedEol => {
                write!(f, "unexpected EOL")
            }
            LexicalError::MalformedInt(pie) => {
                write!(f, "malformed integer number: {pie}")
            }
            LexicalError::MalformedFloat(pie) => {
                write!(f, "malformed float number: {pie}")
            }
            LexicalError::UnterminatedString => {
                write!(f, "unterminated string")
            }
            LexicalError::UnterminatedAnnotation => {
                write!(f, "unterminated annotation")
            }
        }
    }
}
