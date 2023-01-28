use std::{
    fmt,
    num::{ParseFloatError, ParseIntError},
};

use crate::lexer::token::Token;

// #TODO lexer, parser, resolver, etc should be able to return multiple errors
// #TODO maybe just use _one_ Error?
// #TODO think about how to handle Ranged
// #TODO maybe use Ann instead of Ranged?
// #TODO maybe use Expr for the errors?

// #Insight
// Instead of using &'a lifetimes we use `std::mem::take` to extract the errors vec
// from processors.

/// Lexical errors.
#[derive(Debug)]
pub enum LexError {
    UnexpectedEnd,
    MalformedInt(ParseIntError),
    MalformedFloat(ParseFloatError),
    UnterminatedString,
    UnterminatedAnnotation,
}

impl std::error::Error for LexError {}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Self::UnexpectedEnd => "unexpected end of input".to_owned(),
            Self::MalformedInt(pie) => format!("malformed integer number: {pie}"),
            Self::MalformedFloat(pie) => format!("malformed float number: {pie}"),
            Self::UnterminatedString => "unterminated string".to_owned(),
            Self::UnterminatedAnnotation => "unterminated annotation".to_owned(),
        };

        write!(f, "{err}")
    }
}

/// Syntactic errors.
#[derive(Debug)]
pub enum ParseError {
    InvalidQuote,
    UnexpectedToken(Token),
    UnterminatedList,
    MalformedAnnotation(String),
}

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Self::InvalidQuote => "invalid quote".to_owned(),
            Self::UnexpectedToken(token) => format!("unexpected `{token}`"),
            Self::UnterminatedList => "unterminated list".to_owned(),
            Self::MalformedAnnotation(ann) => format!("malformed annotation `{ann}`"),
        };

        write!(f, "{err}")
    }
}

/// Semantic errors.
#[derive(Debug)]
pub enum SemaError {
    UndefinedSymbol(String), // #TODO maybe pass the whole Symbol expression?
    UndefinedFunction(String, String), // #TODO maybe pass the whole Symbol expression?
    InvalidArguments(String),
    NotInvocable(String), // #TODO maybe the non-invocable Annotated<Expr> should be the param?
}

impl std::error::Error for SemaError {}

impl fmt::Display for SemaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Self::UndefinedSymbol(sym) => format!("`{sym}` is undefined"),
            Self::UndefinedFunction(sym, signature) => {
                format!("function `{sym}` with signature `{signature}` is undefined")
            }
            Self::InvalidArguments(text) => text.to_owned(),
            Self::NotInvocable(text) => text.to_owned(),
        };

        write!(f, "{err}")
    }
}

impl SemaError {
    pub fn invalid_arguments(text: impl Into<String>) -> Self {
        Self::InvalidArguments(text.into())
    }

    pub fn not_invocable(text: impl Into<String>) -> Self {
        Self::NotInvocable(text.into())
    }
}

// #TODO is the ComptimeError really worth it?
// #TODO consider renaming this to ComptimeFailure (as it includes multiple errors)

// #[derive(Debug)]
// pub enum ComptimeError {
// }

// impl std::error::Error for ComptimeError {}

// impl fmt::Display for ComptimeError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "ComptimeError")
//     }
// }

// #Insight
// Eval always returns one error.

#[derive(Debug)]
pub enum EvalError {
    Io(std::io::Error),
}

impl std::error::Error for EvalError {}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Self::Io(io_err) => format!("i/o error: {io_err}"),
        };

        write!(f, "{err}")
    }
}

impl From<std::io::Error> for EvalError {
    fn from(value: std::io::Error) -> Self {
        EvalError::Io(value)
    }
}

#[derive(Debug)]
pub enum Error {
    Lex(Vec<LexError>),
    Parse(Vec<ParseError>),
    Sema(Vec<SemaError>),
    Eval(EvalError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lex(errors) => {
                for error in errors {
                    write!(f, "{error}")?;
                }
                write!(f, "")
            }
            Error::Parse(errors) => {
                for error in errors {
                    write!(f, "{error}")?;
                }
                write!(f, "")
            }
            Error::Sema(errors) => {
                for error in errors {
                    write!(f, "{error}")?;
                }
                write!(f, "")
            }
            Error::Eval(error) => write!(f, "{error}"),
        }
    }
}
