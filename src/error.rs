use std::error::Error;
use std::fmt;

use crate::eval::error::EvalError;
use crate::lexer::error::LexicalError;
use crate::parser::error::ParseError;

#[derive(Debug)]
pub enum LanguageError {
    Lexical(LexicalError),
    Parse(ParseError),
    Eval(EvalError),
}

impl Error for LanguageError {}

impl fmt::Display for LanguageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LanguageError::Lexical(err) => err.fmt(f),
            LanguageError::Parse(err) => err.fmt(f),
            LanguageError::Eval(err) => err.fmt(f),
        }
    }
}

impl From<LexicalError> for LanguageError {
    fn from(value: LexicalError) -> Self {
        LanguageError::Lexical(value)
    }
}

impl From<ParseError> for LanguageError {
    fn from(value: ParseError) -> Self {
        LanguageError::Parse(value)
    }
}

impl From<EvalError> for LanguageError {
    fn from(value: EvalError) -> Self {
        LanguageError::Eval(value)
    }
}
