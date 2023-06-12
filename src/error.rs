use std::{
    fmt,
    num::{ParseFloatError, ParseIntError},
};

use crate::{lexer::token::Token, range::Range};

// #insight The implementation is based on https://doc.rust-lang.org/std/io/struct.Error.html

// #TODO: Split comptime/runtime errors?

// #TODO lexer, parser, resolver, etc should be able to return multiple errors
// #TODO maybe just use _one_ Error?
// #TODO think about how to handle Ranged
// #TODO maybe use Ann instead of Ranged?
// #TODO maybe use Expr for the errors?

// #Insight Eval always returns one error, actually no it can call read/parse/ that can return many errors!

#[derive(Debug)]
pub enum ErrorKind {
    // Lexical errors
    UnexpectedEnd,
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
    UndefinedFunction(String, String), // #TODO maybe pass the whole Symbol expression?
    InvalidArguments(String),
    NotInvocable(String), // #TODO maybe the non-invocable Annotated<Expr> should be the param?
    FailedUse,            // #TODO temp, better name needed, rethink!

    // Runtime errors
    Io(std::io::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            ErrorKind::UnexpectedEnd => "unexpected end of input".to_owned(),
            ErrorKind::MalformedInt(pie) => format!("malformed integer number: {pie}"),
            ErrorKind::MalformedFloat(pie) => format!("malformed float number: {pie}"),
            ErrorKind::UnterminatedString => "unterminated string".to_owned(),
            ErrorKind::UnterminatedAnnotation => "unterminated annotation".to_owned(),
            ErrorKind::InvalidQuote => "invalid quote".to_owned(),
            ErrorKind::UnexpectedToken(token) => format!("unexpected `{token}`"),
            ErrorKind::UnterminatedList => "unterminated list".to_owned(),
            ErrorKind::MalformedAnnotation(ann) => format!("malformed annotation `{ann}`"),
            ErrorKind::UndefinedSymbol(sym) => format!("`{sym}` is undefined"),
            ErrorKind::UndefinedFunction(sym, signature) => {
                format!("function `{sym}` with signature `{signature}` is undefined")
            }
            ErrorKind::Io(io_err) => format!("i/o error: {io_err}"),
            ErrorKind::FailedUse => "failed use".to_owned(),
            ErrorKind::InvalidArguments(text) => text.to_owned(),
            ErrorKind::NotInvocable(text) => text.to_owned(),
        };

        write!(f, "{err}")
    }
}

#[derive(Debug)]
pub struct Error {
    /// The kind of the error.
    kind: ErrorKind,
    /// The source text where the error occurred.
    source: Option<String>,
    /// The range of the error, within the source.
    range: Option<Range>,
    // notes: Option<String>,
    // hint: Option<String>,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // #TODO write more information!
        write!(f, "{}", self.kind)
    }
}

impl Error {
    pub fn new(kind: ErrorKind, source: Option<String>, range: Option<Range>) -> Self {
        Self {
            kind,
            source,
            range,
        }
    }

    // #TODO add helper constructors.

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn source(&self) -> &Option<String> {
        &self.source
    }

    pub fn range(&self) -> &Option<Range> {
        &self.range
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::new(ErrorKind::Io(value), None, None)
    }
}
