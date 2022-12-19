use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum LexicalError {
    UnexpectedEol,
    NumberError(ParseIntError),
    UnterminatedStringError,
    UnterminatedAnnotationError,
}

impl Error for LexicalError {}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexicalError::UnexpectedEol => {
                write!(f, "unexpected EOL")
            }
            LexicalError::NumberError(pie) => {
                write!(f, "malformed number: {pie}")
            }
            LexicalError::UnterminatedStringError => {
                write!(f, "unterminated string")
            }
            LexicalError::UnterminatedAnnotationError => {
                write!(f, "unterminated annotation")
            }
        }
    }
}
