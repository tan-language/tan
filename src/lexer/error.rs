use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

use crate::span::Spanned;

// #TODO Consider just having a single `ParseError` or `SyntaxError`?

#[derive(Debug)]
pub enum LexicalError {
    NumberError(ParseIntError),
    UnterminatedStringError,
    UnterminatedAnnotationError,
}

impl Error for LexicalError {}

impl Error for Spanned<LexicalError> {}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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
