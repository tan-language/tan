use std::fmt;

use crate::{
    eval::error::EvalError, lexer::error::LexicalError, parser::error::ParseError, range::Ranged,
};

// #TODO think about how to handle Ranged
// #TODO maybe use Ann instead of Ranged?
// #TODO maybe use Expr for the errors?

#[derive(Debug)]
pub enum Error {
    /// Lexical errors
    Lexical(Ranged<LexicalError>),
    /// Syntactic errors
    Parse(Ranged<ParseError>),
    /// Runtime errors
    Eval(EvalError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Lexical(err) => err.0.fmt(f),
            Error::Parse(err) => err.0.fmt(f),
            Error::Eval(err) => err.fmt(f),
        }
    }
}

impl From<Ranged<LexicalError>> for Error {
    fn from(value: Ranged<LexicalError>) -> Self {
        Error::Lexical(value)
    }
}

impl From<Ranged<ParseError>> for Error {
    fn from(value: Ranged<ParseError>) -> Self {
        Error::Parse(value)
    }
}

impl From<EvalError> for Error {
    fn from(value: EvalError) -> Self {
        Error::Eval(value)
    }
}
