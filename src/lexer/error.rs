use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum LexicalError {
    UnexpectedEol,
    MalformedNumber(ParseIntError),
    UnterminatedString,
    UnterminatedAnnotation,
}

impl Error for LexicalError {}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexicalError::UnexpectedEol => {
                write!(f, "unexpected EOL")
            }
            LexicalError::MalformedNumber(pie) => {
                write!(f, "malformed number: {pie}")
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
