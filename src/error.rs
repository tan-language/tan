use std::fmt;

use crate::{
    eval::error::EvalError,
    lexer::error::LexicalError,
    parser::error::ParseError,
    range::{Range, Ranged},
};

// #TODO maybe just use _one_ Error?
// #TODO think about how to handle Ranged
// #TODO maybe use Ann instead of Ranged?
// #TODO maybe use Expr for the errors?

#[derive(Debug)]
pub enum Error {
    /// Lexical errors
    Lexical(LexicalError),
    /// Syntactic errors
    Parse(ParseError),
    /// Runtime errors
    Eval(EvalError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Lexical(err) => err.fmt(f),
            Error::Parse(err) => err.fmt(f),
            Error::Eval(err) => err.fmt(f),
        }
    }
}

impl From<Ranged<LexicalError>> for Ranged<Error> {
    fn from(value: Ranged<LexicalError>) -> Self {
        Ranged(Error::Lexical(value.0), value.1)
    }
}

impl From<Ranged<ParseError>> for Ranged<Error> {
    fn from(value: Ranged<ParseError>) -> Self {
        Ranged(Error::Parse(value.0), value.1)
    }
}

// #TODO temp, should convert from Ranged<EvalError>
impl From<EvalError> for Ranged<Error> {
    fn from(value: EvalError) -> Self {
        Ranged(Error::Eval(value), Range::default())
    }
}
