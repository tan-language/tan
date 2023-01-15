use std::{
    fmt,
    num::{ParseFloatError, ParseIntError},
};

use crate::{lexer::token::Token, range::Ranged};

// #TODO maybe just use _one_ Error?
// #TODO think about how to handle Ranged
// #TODO maybe use Ann instead of Ranged?
// #TODO maybe use Expr for the errors?

#[derive(Debug)]
pub enum Error {
    // Lexical errors
    UnexpectedEol,
    MalformedInt(ParseIntError),
    MalformedFloat(ParseFloatError),
    UnterminatedString,
    UnterminatedAnnotation,

    // Syntactic (parse) errors
    InvalidQuote,
    UnexpectedToken(Token),
    UnterminatedList,
    MalformedAnnotation(String),

    // Semantic errors
    UndefinedSymbol(String), // #TODO maybe pass the whole Symbol expression?
    InvalidArguments(String),
    NotInvocable(String), // #TODO maybe the non-invocable Annotated<Expr> should be the param?

    // Runtime errors
    Io(std::io::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Error::UnexpectedEol => "unexpected EOL".to_owned(),
            Error::MalformedInt(pie) => format!("malformed integer number: {pie}"),
            Error::MalformedFloat(pie) => format!("malformed float number: {pie}"),
            Error::UnterminatedString => "unterminated string".to_owned(),
            Error::UnterminatedAnnotation => "unterminated annotation".to_owned(),
            Error::InvalidQuote => "invalid quote".to_owned(),
            Error::UnexpectedToken(token) => format!("unexpected `{token}`"),
            Error::UnterminatedList => "unterminated list".to_owned(),
            Error::MalformedAnnotation(ann) => format!("malformed annotation `{ann}`"),
            Error::UndefinedSymbol(sym) => format!("`{sym}` is undefined"),
            Error::Io(io_err) => format!("i/o error: {io_err}"),
            Error::InvalidArguments(text) => text.to_owned(),
            Error::NotInvocable(text) => text.to_owned(),
        };

        write!(f, "{err}")
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl Error {
    pub fn invalid_arguments(text: impl Into<String>) -> Self {
        Self::InvalidArguments(text.into())
    }

    pub fn not_invocable(text: impl Into<String>) -> Self {
        Self::NotInvocable(text.into())
    }
}

impl From<Error> for Ranged<Error> {
    fn from(value: Error) -> Self {
        // #TODO think about this.
        Ranged(value, 0..1)
    }
}